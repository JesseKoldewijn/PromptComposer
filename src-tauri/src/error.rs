use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComposeError {
    #[error("{message}")]
    InvalidInput {
        code: String,
        message: String,
        suggestion: Option<String>,
    },
    #[error("catalog error: {0}")]
    Catalog(String),
}

impl ComposeError {
    pub fn invalid(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            code: code.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    pub fn invalid_suggest(
        code: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::InvalidInput {
            code: code.into(),
            message: message.into(),
            suggestion: Some(suggestion.into()),
        }
    }
}

impl Serialize for ComposeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        match self {
            ComposeError::InvalidInput {
                code,
                message,
                suggestion,
            } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("code", code)?;
                map.serialize_entry("message", message)?;
                map.serialize_entry("suggestion", suggestion)?;
                map.end()
            }
            ComposeError::Catalog(message) => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("code", "catalog")?;
                map.serialize_entry("message", message)?;
                map.serialize_entry("suggestion", &Option::<String>::None)?;
                map.end()
            }
        }
    }
}
