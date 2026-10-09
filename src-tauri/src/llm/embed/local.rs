use crate::llm::{EmbedInput, EmbeddingProvider, LlmError};
use crate::models::{ModelSpec, PoolingStrategy};
use async_trait::async_trait;
use fastembed::{InitOptionsUserDefined, Pooling, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel};
use std::path::PathBuf;
use std::sync::Arc;

/// Maximum texts to pass to fastembed in a single call.
/// Must stay below fastembed's DEFAULT_BATCH_SIZE (256) so par_chunks
/// produces exactly one chunk, avoiding concurrent session.run() calls.
const BATCH_SIZE: usize = 128;

pub struct LocalEmbedder {
    model_id: String,
    dimensions: usize,
    query_prefix: String,
    document_prefix: String,
    model: Arc<TextEmbedding>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore = "Requires model files in KOMODO_TEST_MODELS"]
    async fn downloaded_models_produce_valid_embeddings() {
        let root = PathBuf::from(std::env::var("KOMODO_TEST_MODELS").unwrap());
        for id in ["bge-small-en-v1.5", "multilingual-e5-small"] {
            let spec = crate::models::spec(id).unwrap();
            let embedder = LocalEmbedder::new(root.join(id), spec).unwrap();
            let vectors = embedder
                .embed(
                    &[
                        "A PDF reading assistant".into(),
                        "日本語の論文を読む".into(),
                    ],
                    EmbedInput::Query,
                )
                .await
                .unwrap();
            assert_eq!(vectors.len(), 2);
            for vector in vectors {
                assert_eq!(vector.len(), spec.dimensions);
                assert!(vector.iter().all(|v| v.is_finite()));
                assert!(vector.iter().map(|v| v * v).sum::<f32>() > 0.9);
            }
        }
    }
}

impl LocalEmbedder {
    pub fn new(model_dir: PathBuf, spec: &ModelSpec) -> anyhow::Result<Self> {
        log::debug!("LocalEmbedder::new loading model '{}' from {:?}", spec.id, model_dir);

        // Verify all required files exist
        let required = [
            "model.onnx",
            "tokenizer.json",
            "config.json",
            "tokenizer_config.json",
            "special_tokens_map.json",
        ];
        for name in &required {
            let path = model_dir.join(name);
            if !path.exists() {
                anyhow::bail!(
                    "Model '{}' is not downloaded — open Settings to download it",
                    spec.id
                );
            }
        }

        let load_start = std::time::Instant::now();

        let onnx_file = std::fs::read(model_dir.join("model.onnx"))?;
        log::debug!("LocalEmbedder read model.onnx ({} bytes)", onnx_file.len());

        let tokenizer_file = std::fs::read(model_dir.join("tokenizer.json"))?;
        let config_file = std::fs::read(model_dir.join("config.json"))?;
        let tokenizer_config_file = std::fs::read(model_dir.join("tokenizer_config.json"))?;
        let special_tokens_map_file = std::fs::read(model_dir.join("special_tokens_map.json"))?;

        let tokenizer_files = TokenizerFiles {
            tokenizer_file,
            config_file,
            special_tokens_map_file,
            tokenizer_config_file,
        };

        let pooling = match spec.pooling {
            PoolingStrategy::Mean => Pooling::Mean,
            PoolingStrategy::Cls => Pooling::Cls,
        };

        let user_model = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files)
            .with_pooling(pooling);

        log::debug!("LocalEmbedder initializing TextEmbedding...");
        let model = TextEmbedding::try_new_from_user_defined(
            user_model,
            InitOptionsUserDefined::new(),
        )?;
        log::info!(
            "LocalEmbedder model '{}' loaded in {:?}",
            spec.id,
            load_start.elapsed()
        );

        Ok(Self {
            model_id: format!("local:{}", spec.id),
            dimensions: spec.dimensions,
            query_prefix: spec.query_prefix.to_string(),
            document_prefix: spec.document_prefix.to_string(),
            model: Arc::new(model),
        })
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbedder {
    fn id(&self) -> String {
        self.model_id.clone()
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed(&self, texts: &[String], kind: EmbedInput) -> Result<Vec<Vec<f32>>, LlmError> {
        let prefix = match kind {
            EmbedInput::Document => &self.document_prefix,
            EmbedInput::Query => &self.query_prefix,
        };
        log::debug!(
            "LocalEmbedder::embed start: {} texts, kind={:?}, prefix='{}'",
            texts.len(),
            kind,
            prefix
        );

        let prefixed: Vec<String> = if prefix.is_empty() {
            texts.to_vec()
        } else {
            texts.iter().map(|t| format!("{}{}", prefix, t)).collect()
        };

        let model = Arc::clone(&self.model);

        let embed_start = std::time::Instant::now();
        let total = prefixed.len();

        let embeddings = tokio::task::spawn_blocking(move || {
            let mut all = Vec::with_capacity(total);
            let chunks: Vec<Vec<String>> = prefixed
                .chunks(BATCH_SIZE)
                .map(|c| c.to_vec())
                .collect();
            let num_chunks = chunks.len();

            log::debug!(
                "LocalEmbedder::embed processing {} texts in {} batches (max {} each)",
                total,
                num_chunks,
                BATCH_SIZE
            );

            for (i, batch) in chunks.into_iter().enumerate() {
                let batch_start = std::time::Instant::now();
                log::debug!(
                    "LocalEmbedder::embed batch {}/{}: {} texts — calling fastembed...",
                    i + 1,
                    num_chunks,
                    batch.len()
                );

                let result = model.embed(batch, None);

                match result {
                    Ok(vecs) => {
                        log::debug!(
                            "LocalEmbedder::embed batch {}/{} done: {} vectors in {:?}",
                            i + 1,
                            num_chunks,
                            vecs.len(),
                            batch_start.elapsed()
                        );
                        all.extend(vecs);
                    }
                    Err(e) => {
                        log::error!(
                            "LocalEmbedder::embed batch {}/{} failed: {}",
                            i + 1,
                            num_chunks,
                            e
                        );
                        return Err(e);
                    }
                }
            }

            Ok(all)
        })
        .await
        .map_err(|e| LlmError::Provider(format!("Embedding task panicked: {}", e)))?
        .map_err(|e| LlmError::Provider(format!("fastembed error: {}", e)))?;

        log::info!(
            "LocalEmbedder::embed done: {} vectors in {:?}",
            embeddings.len(),
            embed_start.elapsed()
        );
        Ok(embeddings)
    }
}
