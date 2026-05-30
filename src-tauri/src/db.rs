use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<Connection>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub chat_base_url: String,
    pub chat_model: String,
    pub embed_backend: String,
    pub embed_model: String,
    pub embed_base_url: Option<String>,
    pub embed_model_remote: Option<String>,
    pub top_k: i64,
    pub persona: String,
    pub configured: bool,
    pub log_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            chat_base_url: "https://api.openai.com/v1".to_string(),
            chat_model: "gpt-4o-mini".to_string(),
            embed_backend: "local".to_string(),
            embed_model: "nomic-embed-text-v1.5".to_string(),
            embed_base_url: None,
            embed_model_remote: None,
            top_k: 6,
            persona: "Explain simply".to_string(),
            configured: false,
            log_level: "info".to_string(),
        }
    }
}

fn db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let mut path = crate::portable::data_dir(app_handle);
    path.push("komodo.sqlite");
    path
}

impl Db {
    pub fn new(app_handle: &tauri::AppHandle) -> anyhow::Result<Self> {
        let path = db_path(app_handle);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        let db = Db {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                file_hash TEXT PRIMARY KEY,
                embed_id TEXT NOT NULL,
                indexed_at INTEGER NOT NULL,
                last_opened_at INTEGER NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                file_hash TEXT NOT NULL,
                page INTEGER NOT NULL,
                text TEXT NOT NULL,
                quads_json TEXT NOT NULL,
                vector BLOB NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chunks_file_hash ON chunks(file_hash)",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn is_indexed(&self, file_hash: &str, embed_id: &str) -> bool {
        let conn = self.conn.lock().unwrap();
        let result: Result<Option<i64>, _> = conn
            .query_row(
                "SELECT 1 FROM documents WHERE file_hash = ?1 AND embed_id = ?2",
                params![file_hash, embed_id],
                |row| row.get(0),
            )
            .optional();
        result.ok().flatten().is_some()
    }

    pub fn mark_indexed(&self, file_hash: &str, embed_id: &str) -> anyhow::Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO documents (file_hash, embed_id, indexed_at, last_opened_at)
             VALUES (?1, ?2, ?3, ?3)",
            params![file_hash, embed_id, now],
        )?;
        Ok(())
    }

    pub fn touch_document(&self, file_hash: &str) -> anyhow::Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE documents SET last_opened_at = ?1 WHERE file_hash = ?2",
            params![now, file_hash],
        )?;
        Ok(())
    }

    pub fn get_settings(&self) -> anyhow::Result<Settings> {
        let conn = self.conn.lock().unwrap();
        let mut settings = Settings::default();

        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "chat_base_url" => settings.chat_base_url = value,
                "chat_model" => settings.chat_model = value,
                "embed_backend" => settings.embed_backend = value,
                "embed_model" => settings.embed_model = value,
                "embed_base_url" => settings.embed_base_url = Some(value),
                "embed_model_remote" => settings.embed_model_remote = Some(value),
                "top_k" => settings.top_k = value.parse().unwrap_or(6),
                "persona" => settings.persona = value,
                "configured" => settings.configured = value == "true",
                "log_level" => settings.log_level = value,
                _ => {}
            }
        }
        Ok(settings)
    }

    pub fn set_settings(&self, settings: &Settings) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let top_k_str = settings.top_k.to_string();
        let configured_str = if settings.configured { "true" } else { "false" };
        let pairs = [
            ("chat_base_url", settings.chat_base_url.as_str()),
            ("chat_model", settings.chat_model.as_str()),
            ("embed_backend", settings.embed_backend.as_str()),
            ("embed_model", settings.embed_model.as_str()),
            ("top_k", top_k_str.as_str()),
            ("persona", settings.persona.as_str()),
            ("configured", configured_str),
            ("log_level", settings.log_level.as_str()),
        ];
        for (k, v) in &pairs {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![k, v],
            )?;
        }
        if let Some(v) = &settings.embed_base_url {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params!["embed_base_url", v],
            )?;
        }
        if let Some(v) = &settings.embed_model_remote {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params!["embed_model_remote", v],
            )?;
        }
        Ok(())
    }

    pub fn store_chunks(&self, file_hash: &str, chunks: &[ChunkRow]) -> anyhow::Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        for c in chunks {
            tx.execute(
                "INSERT OR REPLACE INTO chunks (id, file_hash, page, text, quads_json, vector)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    c.id,
                    file_hash,
                    c.page,
                    c.text,
                    c.quads_json,
                    &c.vector as &[u8],
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_chunks(&self, file_hash: &str) -> anyhow::Result<Vec<ChunkRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, file_hash, page, text, quads_json, vector FROM chunks WHERE file_hash = ?1",
        )?;
        let rows = stmt.query_map(params![file_hash], |row| {
            Ok(ChunkRow {
                id: row.get(0)?,
                file_hash: row.get(1)?,
                page: row.get(2)?,
                text: row.get(3)?,
                quads_json: row.get(4)?,
                vector: row.get(5)?,
            })
        })?;
        let mut chunks = Vec::new();
        for row in rows {
            chunks.push(row?);
        }
        Ok(chunks)
    }

    pub fn delete_document(&self, file_hash: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM chunks WHERE file_hash = ?1", params![file_hash])?;
        conn.execute("DELETE FROM documents WHERE file_hash = ?1", params![file_hash])?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ChunkRow {
    pub id: String,
    pub file_hash: String,
    pub page: u32,
    pub text: String,
    pub quads_json: String,
    pub vector: Vec<u8>,
}
