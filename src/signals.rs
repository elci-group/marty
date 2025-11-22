use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Signal {
    Visit { path: String, ts: i64 },
    Reinforce { path: String, weight: f64, ts: i64 },
    Promote { path: String, ts: i64 },
    Demote { path: String, reason: String, ts: i64 },
    Tag { path: String, tag: String, ts: i64 },
}

impl Signal {
    pub fn timestamp_now() -> i64 {
        chrono::Utc::now().timestamp()
    }
}
