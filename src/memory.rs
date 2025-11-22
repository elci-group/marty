// src/memory.rs

use crate::model::Node;
use crate::signals::Signal;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hotspot {
    pub path: String,
    pub energy: f64,
    pub last_ts: i64,
}

#[derive(Debug, Default)]
pub struct Hotspots {
    pub items: Arc<Mutex<HashMap<String, Hotspot>>>,
}

#[derive(Debug, Default)]
pub struct Beliefs {
    pub nodes: HashMap<String, Node>,
}

#[derive(Debug, Default)]
pub struct Trace {
    pub entries: Vec<Signal>,
}
