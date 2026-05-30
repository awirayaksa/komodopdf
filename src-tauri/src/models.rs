use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub dimensions: usize,
    pub pooling: PoolingStrategy,
    pub query_prefix: &'static str,
    pub document_prefix: &'static str,
    pub files: &'static [ModelFile],
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PoolingStrategy {
    Mean,
    Cls,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFile {
    pub local_name: &'static str,
    pub url: &'static str,
    pub size_bytes: u64,
    pub sha256: Option<&'static str>,
}

pub fn registry() -> &'static [ModelSpec] {
    &[
        ModelSpec {
            id: "bge-small-en-v1.5",
            display_name: "bge-small-en-v1.5 (default, EN, 384-d)",
            dimensions: 384,
            pooling: PoolingStrategy::Cls,
            query_prefix: "",
            document_prefix: "",
            files: &[
                ModelFile {
                    local_name: "model.onnx",
                    url: "https://huggingface.co/Qdrant/bge-small-en-v1.5-onnx/resolve/main/model.onnx",
                    size_bytes: 33_000_000,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer.json",
                    url: "https://huggingface.co/Qdrant/bge-small-en-v1.5-onnx/resolve/main/tokenizer.json",
                    size_bytes: 700_000,
                    sha256: None,
                },
                ModelFile {
                    local_name: "config.json",
                    url: "https://huggingface.co/Qdrant/bge-small-en-v1.5-onnx/resolve/main/config.json",
                    size_bytes: 700,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer_config.json",
                    url: "https://huggingface.co/Qdrant/bge-small-en-v1.5-onnx/resolve/main/tokenizer_config.json",
                    size_bytes: 1_200,
                    sha256: None,
                },
                ModelFile {
                    local_name: "special_tokens_map.json",
                    url: "https://huggingface.co/Qdrant/bge-small-en-v1.5-onnx/resolve/main/special_tokens_map.json",
                    size_bytes: 150,
                    sha256: None,
                },
            ],
        },
        ModelSpec {
            id: "multilingual-e5-small",
            display_name: "multilingual-e5-small (CJK/multilingual)",
            dimensions: 384,
            pooling: PoolingStrategy::Mean,
            query_prefix: "query: ",
            document_prefix: "passage: ",
            files: &[
                ModelFile {
                    local_name: "model.onnx",
                    url: "https://huggingface.co/Qdrant/multilingual-e5-small-onnx/resolve/main/model.onnx",
                    size_bytes: 120_000_000,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer.json",
                    url: "https://huggingface.co/Qdrant/multilingual-e5-small-onnx/resolve/main/tokenizer.json",
                    size_bytes: 700_000,
                    sha256: None,
                },
                ModelFile {
                    local_name: "config.json",
                    url: "https://huggingface.co/Qdrant/multilingual-e5-small-onnx/resolve/main/config.json",
                    size_bytes: 700,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer_config.json",
                    url: "https://huggingface.co/Qdrant/multilingual-e5-small-onnx/resolve/main/tokenizer_config.json",
                    size_bytes: 1_200,
                    sha256: None,
                },
                ModelFile {
                    local_name: "special_tokens_map.json",
                    url: "https://huggingface.co/Qdrant/multilingual-e5-small-onnx/resolve/main/special_tokens_map.json",
                    size_bytes: 150,
                    sha256: None,
                },
            ],
        },
        ModelSpec {
            id: "bge-base-en-v1.5",
            display_name: "bge-base-en-v1.5 (higher quality)",
            dimensions: 768,
            pooling: PoolingStrategy::Cls,
            query_prefix: "",
            document_prefix: "",
            files: &[
                ModelFile {
                    local_name: "model.onnx",
                    url: "https://huggingface.co/Qdrant/bge-base-en-v1.5-onnx/resolve/main/model.onnx",
                    size_bytes: 110_000_000,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer.json",
                    url: "https://huggingface.co/Qdrant/bge-base-en-v1.5-onnx/resolve/main/tokenizer.json",
                    size_bytes: 700_000,
                    sha256: None,
                },
                ModelFile {
                    local_name: "config.json",
                    url: "https://huggingface.co/Qdrant/bge-base-en-v1.5-onnx/resolve/main/config.json",
                    size_bytes: 700,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer_config.json",
                    url: "https://huggingface.co/Qdrant/bge-base-en-v1.5-onnx/resolve/main/tokenizer_config.json",
                    size_bytes: 1_200,
                    sha256: None,
                },
                ModelFile {
                    local_name: "special_tokens_map.json",
                    url: "https://huggingface.co/Qdrant/bge-base-en-v1.5-onnx/resolve/main/special_tokens_map.json",
                    size_bytes: 150,
                    sha256: None,
                },
            ],
        },
        ModelSpec {
            id: "nomic-embed-text-v1.5",
            display_name: "nomic-embed-text v1.5 (quantized, 768-d)",
            dimensions: 768,
            pooling: PoolingStrategy::Mean,
            query_prefix: "search_query: ",
            document_prefix: "search_document: ",
            files: &[
                ModelFile {
                    local_name: "model.onnx",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/onnx/model_quantized.onnx",
                    size_bytes: 140_000_000,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/tokenizer.json",
                    size_bytes: 700_000,
                    sha256: None,
                },
                ModelFile {
                    local_name: "config.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/config.json",
                    size_bytes: 700,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer_config.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/tokenizer_config.json",
                    size_bytes: 1_200,
                    sha256: None,
                },
                ModelFile {
                    local_name: "special_tokens_map.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/special_tokens_map.json",
                    size_bytes: 150,
                    sha256: None,
                },
            ],
        },
        ModelSpec {
            id: "nomic-embed-text-v1.5-f32",
            display_name: "nomic-embed-text v1.5 (full f32, 768-d)",
            dimensions: 768,
            pooling: PoolingStrategy::Mean,
            query_prefix: "search_query: ",
            document_prefix: "search_document: ",
            files: &[
                ModelFile {
                    local_name: "model.onnx",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/onnx/model.onnx",
                    size_bytes: 550_000_000,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/tokenizer.json",
                    size_bytes: 700_000,
                    sha256: None,
                },
                ModelFile {
                    local_name: "config.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/config.json",
                    size_bytes: 700,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer_config.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/tokenizer_config.json",
                    size_bytes: 1_200,
                    sha256: None,
                },
                ModelFile {
                    local_name: "special_tokens_map.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/special_tokens_map.json",
                    size_bytes: 150,
                    sha256: None,
                },
            ],
        },
    ]
}

pub fn spec(id: &str) -> Option<&'static ModelSpec> {
    registry().iter().find(|s| s.id == id)
}

pub fn models_dir(app: &tauri::AppHandle) -> PathBuf {
    let mut path = crate::portable::data_dir(app);
    path.push("models");
    path
}

pub fn model_dir(app: &tauri::AppHandle, id: &str) -> PathBuf {
    let mut path = models_dir(app);
    path.push(id);
    path
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStatus {
    pub name: String,
    pub present: bool,
    pub size: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub id: String,
    pub downloaded: bool,
    pub files: Vec<FileStatus>,
    pub present_bytes: u64,
    pub total_bytes: u64,
}

pub fn model_status(app: &tauri::AppHandle, id: &str) -> anyhow::Result<ModelStatus> {
    let spec = spec(id).ok_or_else(|| anyhow::anyhow!("Unknown model id: {}", id))?;
    let dir = model_dir(app, id);
    let mut files = Vec::new();
    let mut present_bytes = 0u64;
    let total_bytes = spec.files.iter().map(|f| f.size_bytes).sum();

    for f in spec.files {
        let path = dir.join(f.local_name);
        let (present, size) = if path.exists() {
            let meta = std::fs::metadata(&path)?;
            (true, meta.len())
        } else {
            (false, 0)
        };
        if present {
            present_bytes += size;
        }
        files.push(FileStatus {
            name: f.local_name.to_string(),
            present,
            size,
        });
    }

    let downloaded = spec.files.iter().all(|f| dir.join(f.local_name).exists());

    Ok(ModelStatus {
        id: id.to_string(),
        downloaded,
        files,
        present_bytes,
        total_bytes,
    })
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub done_bytes: u64,
    pub total_bytes: u64,
    pub file: String,
}

pub async fn download_model(
    app: &tauri::AppHandle,
    id: &str,
    progress: &tauri::ipc::Channel<DownloadProgress>,
) -> anyhow::Result<()> {
    let spec = spec(id).ok_or_else(|| anyhow::anyhow!("Unknown model id: {}", id))?;
    let dir = model_dir(app, id);
    std::fs::create_dir_all(&dir)?;

    let total_bytes = spec.files.iter().map(|f| f.size_bytes).sum::<u64>();
    let mut done_bytes = 0u64;

    for file in spec.files {
        let dest = dir.join(file.local_name);
        if dest.exists() {
            let meta = std::fs::metadata(&dest)?;
            done_bytes += meta.len();
            progress.send(DownloadProgress {
                done_bytes,
                total_bytes,
                file: file.local_name.to_string(),
            })?;
            continue;
        }

        let part = dir.join(format!("{}.part", file.local_name));
        let mut out = std::fs::File::create(&part)?;

        let response = reqwest::get(file.url).await?;
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            std::io::Write::write_all(&mut out, &chunk)?;
            done_bytes += chunk.len() as u64;
            progress.send(DownloadProgress {
                done_bytes,
                total_bytes,
                file: file.local_name.to_string(),
            })?;
        }
        drop(out);

        // Verify sha256 if provided
        if let Some(expected) = file.sha256 {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            let data = std::fs::read(&part)?;
            hasher.update(&data);
            let result = hex::encode(hasher.finalize());
            if result != expected {
                std::fs::remove_file(&part)?;
                anyhow::bail!(
                    "SHA256 mismatch for {}: expected {} got {}",
                    file.local_name,
                    expected,
                    result
                );
            }
        }

        std::fs::rename(&part, &dest)?;
    }

    Ok(())
}

pub fn cleanup_partials(app: &tauri::AppHandle, id: &str) -> anyhow::Result<()> {
    let dir = model_dir(app, id);
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".part") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    Ok(())
}
