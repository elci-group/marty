use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::memory::Hotspot;
use crate::memory::{Beliefs, Hotspots, Trace};
use crate::model::Node;
use crate::signals::Signal;

const STATE_VERSION: u32 = 1;
const MAX_HOTSPOTS: usize = 100;
const MAX_TRACE_ENTRIES: usize = 500;
const MAX_BELIEFS: usize = 1000;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct State {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub hotspots: HashMap<String, Hotspot>,
    #[serde(default)]
    pub beliefs: HashMap<String, Node>,
    #[serde(default)]
    pub trace: Vec<Signal>,
}

/// Default location for persisted state: `~/.marty/state.json`.
pub fn default_state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".marty")
        .join("state.json")
}

/// Load state from disk into the in-memory models. Missing or corrupt files are
/// ignored so the CLI always starts with a usable (possibly empty) state.
pub fn load(
    path: &Path,
    hotspots: &Arc<Mutex<Hotspots>>,
    beliefs: &Arc<Mutex<Beliefs>>,
    trace: &Arc<Mutex<Trace>>,
) {
    if !path.exists() {
        return;
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Warning: could not read state file {}: {}",
                path.display(),
                e
            );
            return;
        }
    };

    let mut state: State = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Warning: state file {} is corrupt ({}); starting fresh.",
                path.display(),
                e
            );
            return;
        }
    };

    // If the on-disk version is older than the current format, we continue with
    // what we can read and rewrite the file in the new format on the next save.
    if state.version > 0 && state.version != STATE_VERSION {
        tracing::info!(
            file_version = state.version,
            current_version = STATE_VERSION,
            "migrating marty state file"
        );
    }

    hotspots.lock().items = Arc::new(Mutex::new(state.hotspots));
    beliefs.lock().nodes = state.beliefs;
    trace.lock().entries = std::mem::take(&mut state.trace);
}

/// Save the current in-memory models to disk atomically.
pub fn save(
    path: &Path,
    hotspots: &Arc<Mutex<Hotspots>>,
    beliefs: &Arc<Mutex<Beliefs>>,
    trace: &Arc<Mutex<Trace>>,
) -> Result<(), String> {
    let mut state = State {
        version: 0,
        hotspots: hotspots.lock().items.lock().clone(),
        beliefs: beliefs.lock().nodes.clone(),
        trace: trace.lock().entries.clone(),
    };

    normalize_state(&mut state);

    let json = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;

    // Restrict read access to the owner; the state may contain sensitive paths.
    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }

    std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;

    Ok(())
}

/// Enforce size caps and set the current schema version before writing.
fn normalize_state(state: &mut State) {
    state.version = STATE_VERSION;

    if state.hotspots.len() > MAX_HOTSPOTS {
        let mut items: Vec<(String, Hotspot)> = state.hotspots.drain().collect();
        items.sort_by(|a, b| {
            b.1.energy
                .partial_cmp(&a.1.energy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        state.hotspots = items.into_iter().take(MAX_HOTSPOTS).collect();
    }

    if state.trace.len() > MAX_TRACE_ENTRIES {
        let start = state.trace.len() - MAX_TRACE_ENTRIES;
        state.trace = state.trace.split_off(start);
    }

    if state.beliefs.len() > MAX_BELIEFS {
        let mut items: Vec<(String, Node)> = state.beliefs.drain().collect();
        items.truncate(MAX_BELIEFS);
        state.beliefs = items.into_iter().collect();
    }
}
