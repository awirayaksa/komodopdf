mod db;
mod llm;
mod models;
mod open_requests;
#[cfg(target_os = "windows")]
mod launch_dispatcher;
mod portable;
mod rag;
mod secrets;

use db::{Db, Settings};
use llm::{ChatEvent, EmbedInput, LlmProvider, Quad};
use rag::{build_grounding, index_document, IndexProgress, PageExtraction, ReaderContext};
use secrets::KeySlot;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::{Emitter, Manager};
use tauri_plugin_fs::FsExt;
use futures_util::StreamExt;

fn log_level_filter(level: &str) -> log::LevelFilter {
    match level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    }
}

struct AppLogger {
    file: Option<Mutex<File>>,
}

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let msg = format!("[{} {}] {}", record.level(), record.target(), record.args());
        eprintln!("{}", msg);
        if let Some(file) = &self.file {
            let mut f = file.lock().unwrap();
            let _ = writeln!(f, "{}", msg);
            let _ = f.flush();
        }
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
        if let Some(file) = &self.file {
            let _ = file.lock().unwrap().flush();
        }
    }
}

fn init_logger(initial_level: &str, log_dir: Option<PathBuf>) {
    let log_file = log_dir
        .map(|dir| {
            let path = dir.join("komodo.log");
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .ok()
                .map(Mutex::new)
        })
        .flatten();

    let logger = AppLogger { file: log_file };
    log::set_boxed_logger(Box::new(logger)).expect("failed to set logger");
    set_log_level(initial_level);
}

fn set_log_level(level: &str) {
    let filter = log_level_filter(level);
    log::set_max_level(filter);
}

struct LlmLogger {
    file: Mutex<File>,
    system_logged: Mutex<HashMap<String, bool>>,
}

impl LlmLogger {
    fn new(path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            file: Mutex::new(file),
            system_logged: Mutex::new(HashMap::new()),
        })
    }

    fn log_request(&self, file_hash: &str, system_prompt: &str, user_message: &str, is_first: bool) {
        let mut state = self.system_logged.lock().unwrap();
        let mut file = self.file.lock().unwrap();

        if is_first || !state.get(file_hash).copied().unwrap_or(false) {
            state.insert(file_hash.to_string(), true);
            let _ = writeln!(file, "system prompt: {}", system_prompt);
        }
        let _ = writeln!(file, "user: {}", user_message);
        let _ = file.flush();
    }

    fn log_response(&self, _file_hash: &str, response: &str) {
        let mut file = self.file.lock().unwrap();
        let _ = writeln!(file, "result: {}", response);
        let _ = file.flush();
    }

    #[allow(dead_code)]
    fn reset_session(&self, file_hash: &str) {
        let mut state = self.system_logged.lock().unwrap();
        state.remove(file_hash);
    }
}

#[tauri::command]
fn set_api_key(slot: KeySlot, key: String) -> Result<(), String> {
    secrets::set_api_key(slot, key)
}

#[tauri::command]
fn has_api_key(slot: KeySlot) -> bool {
    secrets::has_api_key(slot)
}

#[tauri::command]
fn delete_api_key(slot: KeySlot) -> Result<(), String> {
    secrets::delete_api_key(slot)
}

#[tauri::command]
fn is_indexed(state: tauri::State<'_, Arc<Db>>, file_hash: String, embed_id: String) -> bool {
    state.is_indexed(&file_hash, &rag::index_id(&embed_id))
}

#[tauri::command]
fn delete_document(state: tauri::State<'_, Arc<Db>>, file_hash: String) -> Result<(), String> {
    state.delete_document(&file_hash).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, Arc<Db>>) -> Result<Settings, String> {
    state.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_settings(state: tauri::State<'_, Arc<Db>>, settings: Settings) -> Result<(), String> {
    state.set_settings(&settings).map_err(|e| e.to_string())?;
    set_log_level(&settings.log_level);
    Ok(())
}

#[tauri::command]
fn is_audit_enabled(app: tauri::AppHandle) -> bool {
    let path = crate::portable::data_dir(&app).join("debug.json");
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let map: serde_json::Map<String, serde_json::Value> = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(_) => return false,
    };
    map.get("auditPdfExtract")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[tauri::command]
fn write_audit_file(app: tauri::AppHandle, file_hash: String, content: String) -> Result<(), String> {
    let dir = crate::portable::data_dir(&app).join("pdf-audit");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path = dir.join(format!("{}_{}.txt", file_hash, timestamp));
    let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn index_document_cmd(
    state: tauri::State<'_, Arc<Db>>,
    app: tauri::AppHandle,
    file_hash: String,
    pages: Vec<PageExtraction>,
    progress: Channel<IndexProgress>,
) -> Result<(), String> {
    log::info!("index_document_cmd start: file_hash={}", file_hash);
    let settings = state.get_settings().map_err(|e| e.to_string())?;
    log::debug!(
        "index settings: backend={} model={}",
        settings.embed_backend,
        settings.embed_model
    );

    let embedder: Box<dyn llm::EmbeddingProvider> = if settings.embed_backend == "remote" {
        let base_url = settings.embed_base_url.unwrap_or_default();
        let model = settings.embed_model_remote.unwrap_or_default();
        let key = secrets::get_api_key(KeySlot::Embed)
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        log::debug!("using remote embedder: base_url={} model={}", base_url, model);
        Box::new(llm::embed::openai_compat::OpenAiCompatibleEmbedder::new(
            base_url, key, model, 1536,
        ))
    } else {
        let spec = models::spec(&settings.embed_model)
            .ok_or_else(|| format!("Unknown embedding model: {}", settings.embed_model))?;
        let dir = models::model_dir(&app, &settings.embed_model);
        log::debug!("using local embedder: model_id={} dir={:?}", spec.id, dir);
        Box::new(
            llm::embed::local::LocalEmbedder::new(dir, spec)
                .map_err(|e| e.to_string())?,
        )
    };
    log::debug!("embedder created, dimensions={}", embedder.dimensions());

    let result = index_document(&*state, &file_hash, pages, embedder.as_ref(), progress)
        .await
        .map_err(|e| e.to_string());
    if let Err(ref e) = result {
        log::error!("index_document failed: {}", e);
    } else {
        log::info!("index_document_cmd finished successfully");
    }
    result
}

#[tauri::command]
async fn ask(
    state: tauri::State<'_, Arc<Db>>,
    app: tauri::AppHandle,
    file_hash: String,
    question: String,
    current_page: u32,
    selection_text: Option<String>,
    selection_quads: Vec<Quad>,
    channel: Channel<ChatEvent>,
    message_count: usize,
    history: Vec<llm::ChatMessage>,
    persona: String,
) -> Result<(), String> {
    let settings = state.get_settings().map_err(|e| e.to_string())?;

    let ctx = ReaderContext {
        current_page,
        selection_text,
        selection_quads,
    };

    let embedder: Box<dyn llm::EmbeddingProvider> = if settings.embed_backend == "remote" {
        let base_url = settings.embed_base_url.clone().unwrap_or_default();
        let model = settings.embed_model_remote.clone().unwrap_or_default();
        let key = secrets::get_api_key(KeySlot::Embed)
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        Box::new(llm::embed::openai_compat::OpenAiCompatibleEmbedder::new(
            base_url, key, model, 1536,
        ))
    } else {
        let spec = models::spec(&settings.embed_model)
            .ok_or_else(|| format!("Unknown embedding model: {}", settings.embed_model))?;
        let dir = models::model_dir(&app, &settings.embed_model);
        Box::new(
            llm::embed::local::LocalEmbedder::new(dir, spec)
                .map_err(|e| e.to_string())?,
        )
    };

    let top_k = settings.top_k as usize;
    if !state.is_indexed(&file_hash, &rag::index_id(&embedder.id())) {
        return Err("This PDF needs to be reindexed with the current embedding settings".into());
    }
    let grounding = build_grounding(
        &*state, &file_hash, &question, &ctx, top_k, embedder.as_ref(), &persona,
    )
    .await
    .map_err(|e| e.to_string())?;

    channel
        .send(ChatEvent::Sources(grounding.sources))
        .map_err(|e| e.to_string())?;

    let chat_key = secrets::get_api_key(KeySlot::Chat)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let provider = llm::chat::openai_compat::OpenAiCompatibleProvider::new(
        settings.chat_base_url,
        chat_key,
        settings.chat_model.clone(),
    );

    let model = settings.chat_model.clone();
    let is_first = message_count == 0;
    let llm_logger = app.try_state::<LlmLogger>();

    // Confidence routing: pick prompt based on max retrieval score.
    // Threshold tuned conservatively; high score → grounded prompt, low → general knowledge.
    let score_threshold: f32 = 0.55;
    let use_rag = grounding.max_score >= score_threshold;

    let system_prompt = if use_rag {
        grounding.system_prompt.clone()
    } else {
        grounding.knowledge_system_prompt.clone()
    };

    let mut messages = history;
    messages.push(llm::ChatMessage {
        role: llm::Role::User,
        content: grounding.user_prompt,
    });

    let req = llm::ChatRequest {
        model,
        messages,
        system_prompt: system_prompt.clone(),
        temperature: Some(0.2),
    };

    if let Some(logger) = &llm_logger {
        logger.log_request(&file_hash, &system_prompt, &req.messages.last().map(|m| m.content.as_str()).unwrap_or(""), is_first);
    }

    // Stream immediately in both cases — no double generation, no judge pause.
    if !use_rag {
        let note = "> _The document didn't contain a strong answer here, so this draws mainly on general knowledge._\n\n";
        channel
            .send(ChatEvent::Token(note.to_string()))
            .map_err(|e| e.to_string())?;
    }

    let mut stream = provider.chat_stream(req).await.map_err(|e| e.to_string())?;
    let mut full_response = String::new();
    while let Some(token_result) = stream.next().await {
        match token_result {
            Ok(token) => {
                full_response.push_str(&token);
                channel
                    .send(ChatEvent::Token(token))
                    .map_err(|e| e.to_string())?;
            }
            Err(e) => {
                channel
                    .send(ChatEvent::Error(e.to_string()))
                    .map_err(|e| e.to_string())?;
                break;
            }
        }
    }

    if let Some(logger) = &llm_logger {
        logger.log_response(&file_hash, &full_response);
    }
    channel.send(ChatEvent::Done).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TestResult {
    chat_ok: bool,
    chat_err: Option<String>,
    embed_ok: bool,
    embed_err: Option<String>,
}

#[tauri::command]
async fn test_connection(
    state: tauri::State<'_, Arc<Db>>,
    app: tauri::AppHandle,
    chat_key: Option<String>,
    embed_key: Option<String>,
) -> Result<TestResult, String> {
    let settings = state.get_settings().map_err(|e| e.to_string())?;

    let mut chat_ok = false;
    let mut chat_err = None;

    let key_to_test = chat_key
        .filter(|k| !k.is_empty())
        .or_else(|| secrets::get_api_key(KeySlot::Chat).ok().flatten());

    if let Some(key) = key_to_test {
        let provider = llm::chat::openai_compat::OpenAiCompatibleProvider::new(
            settings.chat_base_url.clone(),
            key,
            settings.chat_model.clone(),
        );
        let req = llm::ChatRequest {
            model: settings.chat_model.clone(),
            messages: vec![llm::ChatMessage {
                role: llm::Role::User,
                content: "hi".to_string(),
            }],
            system_prompt: "You are a helpful assistant.".to_string(),
            temperature: None,
        };
        match provider.chat_stream(req).await {
            Ok(mut stream) => {
                if let Some(Ok(_)) = stream.next().await {
                    chat_ok = true;
                } else {
                    chat_err = Some("No response".to_string());
                }
            }
            Err(e) => {
                chat_err = Some(e.to_string());
            }
        }
    } else {
        chat_err = Some("No API key set".to_string());
    }

    let mut embed_ok = false;
    let mut embed_err = None;

    let embedder: Box<dyn llm::EmbeddingProvider> = if settings.embed_backend == "remote" {
        let base_url = settings.embed_base_url.clone().unwrap_or_default();
        let model = settings.embed_model_remote.clone().unwrap_or_default();
        let key = embed_key
            .filter(|k| !k.is_empty())
            .or_else(|| secrets::get_api_key(KeySlot::Embed).ok().flatten())
            .unwrap_or_default();
        Box::new(llm::embed::openai_compat::OpenAiCompatibleEmbedder::new(
            base_url, key, model, 1536,
        ))
    } else {
        let spec = models::spec(&settings.embed_model)
            .ok_or_else(|| format!("Unknown embedding model: {}", settings.embed_model))?;
        let dir = models::model_dir(&app, &settings.embed_model);
        match llm::embed::local::LocalEmbedder::new(dir, spec) {
            Ok(e) => Box::new(e),
            Err(e) => {
                return Ok(TestResult {
                    chat_ok,
                    chat_err,
                    embed_ok: false,
                    embed_err: Some(e.to_string()),
                });
            }
        }
    };

    match embedder.embed(&["test".to_string()], EmbedInput::Query).await {
        Ok(_) => embed_ok = true,
        Err(e) => embed_err = Some(e.to_string()),
    }

    Ok(TestResult {
        chat_ok,
        chat_err,
        embed_ok,
        embed_err,
    })
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelInfo {
    id: String,
    display_name: String,
    dimensions: usize,
    status: models::ModelStatus,
}

#[tauri::command]
fn list_embedding_models(app: tauri::AppHandle) -> Result<Vec<ModelInfo>, String> {
    let mut out = Vec::new();
    for spec in models::registry() {
        let status = models::model_status(&app, spec.id).map_err(|e| e.to_string())?;
        out.push(ModelInfo {
            id: spec.id.to_string(),
            display_name: spec.display_name.to_string(),
            dimensions: spec.dimensions,
            status,
        });
    }
    Ok(out)
}

#[tauri::command]
fn model_status_cmd(app: tauri::AppHandle, model_id: String) -> Result<models::ModelStatus, String> {
    models::model_status(&app, &model_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_model_cmd(
    app: tauri::AppHandle,
    model_id: String,
    progress: Channel<models::DownloadProgress>,
) -> Result<(), String> {
    let result = models::download_model(&app, &model_id, &progress).await;
    if result.is_err() {
        let _ = models::cleanup_partials(&app, &model_id);
    }
    result.map_err(|e| e.to_string())
}

#[tauri::command]
fn open_models_folder_cmd(app: tauri::AppHandle, model_id: Option<String>) -> Result<(), String> {
    let path = if let Some(id) = model_id {
        models::model_dir(&app, &id)
    } else {
        models::models_dir(&app)
    };
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    tauri_plugin_opener::reveal_item_in_dir(&path).map_err(|e| e.to_string())
}

// The event is a wake-up; the managed queue remains authoritative even before
// the webview (or other plugins) is ready.
fn queue_open_requests(app: &tauri::AppHandle, args: Vec<String>, cwd: &std::path::Path) {
    let paths = open_requests::pdf_paths(args, cwd)
        .into_iter()
        .filter_map(|path| path.to_str().map(str::to_owned))
        .collect::<Vec<_>>();
    if !paths.is_empty() {
        app.state::<open_requests::OpenRequests>().push(paths);
        let _ = app.emit_to("main", "pdf-open-requested", ());
    }
}

#[tauri::command]
fn take_pdf_open_requests(
    app: tauri::AppHandle,
    state: tauri::State<'_, open_requests::OpenRequests>,
) -> Result<Vec<String>, String> {
    let paths = state.drain();
    // Scope only the individual OS-requested files, after all plugins are
    // initialized. No unrestricted directory read permission is needed.
    for path in &paths {
        app.fs_scope().allow_file(path).map_err(|e| e.to_string())?;
    }
    Ok(paths)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().manage(open_requests::OpenRequests::default());
    // Register first so a second process exits before opening the DB or key store.
    #[cfg(target_os = "windows")]
    let builder = {
        let launches = std::sync::OnceLock::<launch_dispatcher::LaunchDispatcher>::new();
        builder.plugin(tauri_plugin_single_instance::init(move |app, args, cwd| {
            let dispatcher = launches.get_or_init(|| {
                let app = app.clone();
                launch_dispatcher::LaunchDispatcher::new(move |request| {
                    queue_open_requests(&app, request.args, std::path::Path::new(&request.cwd));
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                })
            });
            if dispatcher.send(args, cwd).is_err() {
                log::error!("PDF launch worker stopped unexpectedly");
            }
        }))
    };
    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let db = Arc::new(Db::new(app.handle())?);
            let settings = db.get_settings().unwrap_or_default();

            let app_data_dir = crate::portable::data_dir(app);
            init_logger(&settings.log_level, Some(app_data_dir.clone()));

            if let Ok(logger) = LlmLogger::new(app_data_dir.join("llm.log")) {
                app.manage(logger);
            }

            log::info!("Komodo PDF starting — log level: {}", settings.log_level);
            // Initialise the on-disk fallback for API keys so they survive restarts
            // even when the OS keyring is unavailable.
            secrets::init_file_path(app_data_dir.join("secrets.json"));
            app.manage(db);
            if let Ok(cwd) = std::env::current_dir() {
                queue_open_requests(app.handle(), std::env::args().collect(), &cwd);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            take_pdf_open_requests,
            set_api_key,
            has_api_key,
            delete_api_key,
            is_indexed,
            delete_document,
            get_settings,
            set_settings,
            is_audit_enabled,
            write_audit_file,
            index_document_cmd,
            ask,
            test_connection,
            list_embedding_models,
            model_status_cmd,
            download_model_cmd,
            open_models_folder_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
