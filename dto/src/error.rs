use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestError {
    pub message: String,
}

impl From<String> for RestError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for RestError {
    fn from(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}
