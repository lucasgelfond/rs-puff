use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    Uint(u64),
    String(String),
}

impl From<u64> for Id {
    fn from(v: u64) -> Self {
        Id::Uint(v)
    }
}

impl From<i64> for Id {
    fn from(v: i64) -> Self {
        Id::Uint(v as u64)
    }
}

impl From<i32> for Id {
    fn from(v: i32) -> Self {
        Id::Uint(v as u64)
    }
}

impl From<&str> for Id {
    fn from(v: &str) -> Self {
        Id::String(v.to_string())
    }
}

impl From<String> for Id {
    fn from(v: String) -> Self {
        Id::String(v)
    }
}
