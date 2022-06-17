#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to deserialize from or serialize to JSON.")]
    JsonFailed(#[from] serde_json::Error),
}
