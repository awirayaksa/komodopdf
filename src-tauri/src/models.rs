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

// Public ONNX exports pinned to immutable revisions; metadata verified via the HF API.
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
                    url: "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/ea104dacec62c0de699686887e3f920caeb4f3e3/onnx/model_quantized.onnx",
                    size_bytes: 34_014_426,
                    sha256: Some("6c9c6101a956d62dfb5e7190c538226c0c5bb9cb27b651234b6df063ee7dbfe4"),
                },
                ModelFile {
                    local_name: "tokenizer.json",
                    url: "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/ea104dacec62c0de699686887e3f920caeb4f3e3/tokenizer.json",
                    size_bytes: 711_396,
                    sha256: None,
                },
                ModelFile {
                    local_name: "config.json",
                    url: "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/ea104dacec62c0de699686887e3f920caeb4f3e3/config.json",
                    size_bytes: 683,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer_config.json",
                    url: "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/ea104dacec62c0de699686887e3f920caeb4f3e3/tokenizer_config.json",
                    size_bytes: 366,
                    sha256: None,
                },
                ModelFile {
                    local_name: "special_tokens_map.json",
                    url: "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/ea104dacec62c0de699686887e3f920caeb4f3e3/special_tokens_map.json",
                    size_bytes: 125,
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
                    url: "https://huggingface.co/Xenova/multilingual-e5-small/resolve/761b726dd34fb83930e26aab4e9ac3899aa1fa78/onnx/model_quantized.onnx",
                    size_bytes: 118_308_185,
                    sha256: Some("f80102d3f2a1229f387d3c81909990d8945513e347b0eab049f7de3c6f98c193"),
                },
                ModelFile {
                    local_name: "tokenizer.json",
                    url: "https://huggingface.co/Xenova/multilingual-e5-small/resolve/761b726dd34fb83930e26aab4e9ac3899aa1fa78/tokenizer.json",
                    size_bytes: 17_082_730,
                    sha256: Some("0b44a9d7b51c3c62626640cda0e2c2f70fdacdc25bbbd68038369d14ebdf4c39"),
                },
                ModelFile {
                    local_name: "config.json",
                    url: "https://huggingface.co/Xenova/multilingual-e5-small/resolve/761b726dd34fb83930e26aab4e9ac3899aa1fa78/config.json",
                    size_bytes: 658,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer_config.json",
                    url: "https://huggingface.co/Xenova/multilingual-e5-small/resolve/761b726dd34fb83930e26aab4e9ac3899aa1fa78/tokenizer_config.json",
                    size_bytes: 443,
                    sha256: None,
                },
                ModelFile {
                    local_name: "special_tokens_map.json",
                    url: "https://huggingface.co/Xenova/multilingual-e5-small/resolve/761b726dd34fb83930e26aab4e9ac3899aa1fa78/special_tokens_map.json",
                    size_bytes: 167,
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
                    url: "https://huggingface.co/Xenova/bge-base-en-v1.5/resolve/4d6cd88e18e51a5e020c2c305726d76ada9c03cf/onnx/model_quantized.onnx",
                    size_bytes: 110_083_337,
                    sha256: Some("c9729cc84cbd0e9fecc759505d2be65916c9fe05222d7ea26c65fcb3382af38d"),
                },
                ModelFile {
                    local_name: "tokenizer.json",
                    url: "https://huggingface.co/Xenova/bge-base-en-v1.5/resolve/4d6cd88e18e51a5e020c2c305726d76ada9c03cf/tokenizer.json",
                    size_bytes: 711_396,
                    sha256: None,
                },
                ModelFile {
                    local_name: "config.json",
                    url: "https://huggingface.co/Xenova/bge-base-en-v1.5/resolve/4d6cd88e18e51a5e020c2c305726d76ada9c03cf/config.json",
                    size_bytes: 717,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer_config.json",
                    url: "https://huggingface.co/Xenova/bge-base-en-v1.5/resolve/4d6cd88e18e51a5e020c2c305726d76ada9c03cf/tokenizer_config.json",
                    size_bytes: 366,
                    sha256: None,
                },
                ModelFile {
                    local_name: "special_tokens_map.json",
                    url: "https://huggingface.co/Xenova/bge-base-en-v1.5/resolve/4d6cd88e18e51a5e020c2c305726d76ada9c03cf/special_tokens_map.json",
                    size_bytes: 125,
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
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/e9b6763023c676ca8431644204f50c2b100d9aab/onnx/model_quantized.onnx",
                    size_bytes: 137_296_292,
                    sha256: Some("b4342336debaea79de872370664b0aaeb67dea4605513d00ee236ea871a81f27"),
                },
                ModelFile {
                    local_name: "tokenizer.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/e9b6763023c676ca8431644204f50c2b100d9aab/tokenizer.json",
                    size_bytes: 711_396,
                    sha256: None,
                },
                ModelFile {
                    local_name: "config.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/e9b6763023c676ca8431644204f50c2b100d9aab/config.json",
                    size_bytes: 2_538,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer_config.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/e9b6763023c676ca8431644204f50c2b100d9aab/tokenizer_config.json",
                    size_bytes: 1_191,
                    sha256: None,
                },
                ModelFile {
                    local_name: "special_tokens_map.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/e9b6763023c676ca8431644204f50c2b100d9aab/special_tokens_map.json",
                    size_bytes: 695,
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
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/e9b6763023c676ca8431644204f50c2b100d9aab/onnx/model.onnx",
                    size_bytes: 547_310_275,
                    sha256: Some("147d5aa88c2101237358e17796cf3a227cead1ec304ec34b465bb08e9d952965"),
                },
                ModelFile {
                    local_name: "tokenizer.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/e9b6763023c676ca8431644204f50c2b100d9aab/tokenizer.json",
                    size_bytes: 711_396,
                    sha256: None,
                },
                ModelFile {
                    local_name: "config.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/e9b6763023c676ca8431644204f50c2b100d9aab/config.json",
                    size_bytes: 2_538,
                    sha256: None,
                },
                ModelFile {
                    local_name: "tokenizer_config.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/e9b6763023c676ca8431644204f50c2b100d9aab/tokenizer_config.json",
                    size_bytes: 1_191,
                    sha256: None,
                },
                ModelFile {
                    local_name: "special_tokens_map.json",
                    url: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/e9b6763023c676ca8431644204f50c2b100d9aab/special_tokens_map.json",
                    size_bytes: 695,
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
        let (present, size) = if path.is_file() {
            let meta = std::fs::metadata(&path)?;
            (valid_file(&path, f), meta.len())
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

    let downloaded = files.iter().all(|f| f.present);

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

fn valid_file(path: &std::path::Path, spec: &ModelFile) -> bool {
    if std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) != spec.size_bytes {
        return false;
    }
    if spec.local_name.ends_with(".json") {
        return std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .is_some();
    }
    true
}

fn verify_hash(path: &std::path::Path, expected: Option<&str>) -> anyhow::Result<()> {
    if let Some(expected) = expected {
        use sha2::{Digest, Sha256};
        use std::io::Read;
        let mut file = std::fs::File::open(path)?;
        let mut hash = Sha256::new();
        let mut bytes = [0; 65536];
        loop {
            let count = file.read(&mut bytes)?;
            if count == 0 {
                break;
            }
            hash.update(&bytes[..count]);
        }
        anyhow::ensure!(
            hex::encode(hash.finalize()) == expected,
            "SHA256 mismatch for {}",
            path.display()
        );
    }
    Ok(())
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
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(600))
        .build()?;
    let mut last_progress = std::time::Instant::now();

    for file in spec.files {
        let dest = dir.join(file.local_name);
        if valid_file(&dest, file) && verify_hash(&dest, file.sha256).is_ok() {
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

        let response = client.get(file.url).send().await?.error_for_status()?;
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            std::io::Write::write_all(&mut out, &chunk)?;
            done_bytes += chunk.len() as u64;
            if last_progress.elapsed() >= std::time::Duration::from_millis(100) {
                progress.send(DownloadProgress {
                    done_bytes,
                    total_bytes,
                    file: file.local_name.to_string(),
                })?;
                last_progress = std::time::Instant::now();
            }
        }
        out.sync_all()?;
        drop(out);

        anyhow::ensure!(
            valid_file(&part, file),
            "Invalid or incomplete model file: {}",
            file.local_name
        );
        verify_hash(&part, file.sha256)?;

        if dest.exists() {
            std::fs::remove_file(&dest)?;
        }
        std::fs::rename(&part, &dest)?;
        progress.send(DownloadProgress {
            done_bytes,
            total_bytes,
            file: file.local_name.to_string(),
        })?;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_error_pages_and_truncated_downloads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let file = ModelFile {
            local_name: "config.json",
            url: "",
            size_bytes: 2,
            sha256: None,
        };
        std::fs::write(&path, "{}").unwrap();
        assert!(valid_file(&path, &file));
        std::fs::write(&path, "<>").unwrap();
        assert!(!valid_file(&path, &file));
        std::fs::write(&path, "{").unwrap();
        assert!(!valid_file(&path, &file));
    }

    #[test]
    fn checks_model_hash_without_loading_entire_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model.onnx");
        std::fs::write(&path, "abc").unwrap();
        assert!(verify_hash(
            &path,
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        )
        .is_ok());
        std::fs::write(&path, "abd").unwrap();
        assert!(verify_hash(
            &path,
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        )
        .is_err());
    }
}
