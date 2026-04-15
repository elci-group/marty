use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub metadata: HashMap<String, String>,
}

impl Node {
    #[allow(dead_code)]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            metadata: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}
