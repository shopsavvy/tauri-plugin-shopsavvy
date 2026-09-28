use serde::{ser::Serializer, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing API key — pass via Builder::api_key or set SHOPSAVVY_API_KEY env var")]
    MissingApiKey,
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API returned status {0}: {1}")]
    Api(u16, String),
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
