use colored::*;
use crate::memory::{Hotspots, Beliefs, Trace};
use crate::signals::Signal;

use tracing::{info, error};

pub fn success(msg: &str) {
    info!("✅ {}", msg.truecolor(0, 255, 0).bold());
}



pub fn error(msg: &str) {
    error!("❌ {}", msg.truecolor(255, 0, 0).bold());
}



use std::env;

pub fn format_path_with_emojis(path: &str) -> String {
    let home = env::var("HOME").unwrap_or_default();
    let mut result = path.to_string();

    if !home.is_empty() && result.starts_with(&home) {
        result = result.replacen(&home, "🏠", 1);
    }

    result = result.replace("$HOME", "🏠");
    result = result.replace("~", "🏠");

    result
}

pub fn print_visit(path: &str) {
    info!("📍 Visited: {}", format_path_with_emojis(path).truecolor(0, 191, 255));
}

pub fn print_hotspots(hotspots: &Hotspots, top: usize) {
    info!("🔥 Top Hotspots");
    let items = hotspots.items.lock();
    let mut sorted_hotspots: Vec<_> = items.values().collect();
    sorted_hotspots.sort_by(|a, b| b.energy.partial_cmp(&a.energy).unwrap_or(std::cmp::Ordering::Equal));

    if sorted_hotspots.is_empty() {
        info!("No hotspots recorded yet. Use 'marty visit <path>' to start.");
        return;
    }

    for (i, hs) in sorted_hotspots.iter().take(top).enumerate() {
        let rank = match i {
            0 => "🥇".to_string(),
            1 => "🥈".to_string(),
            2 => "🥉".to_string(),
            _ => format!("#{}", i + 1),
        };
        info!(
            "{} {} {} ({})",
            rank,
            format_path_with_emojis(&hs.path).bright_white(),
            "⚡".repeat( (hs.energy / 10.0).ceil() as usize).truecolor(255, 255, 0),
            hs.energy.to_string().truecolor(192, 192, 192)
        );
    }
}

pub fn print_beliefs(beliefs: &Beliefs) {
    info!("🧠 Beliefs Network");
    if beliefs.nodes.is_empty() {
        info!("No beliefs formed yet. Try visiting a few directories.");
        return;
    }
    for (id, node) in &beliefs.nodes {
        info!("🔹 {} ({})", node.label.bright_white().bold(), format_path_with_emojis(&id.to_string()).truecolor(192, 192, 192));
        for (key, value) in &node.metadata {
            info!("  - {}: {}", key.truecolor(0, 191, 255), value.to_string().italic());
        }
    }
}

pub fn print_trace(trace: &Trace, last: usize) {
    info!("📜 Recent Activity Trace");
    if trace.entries.is_empty() {
        info!("No activity recorded yet. Get started with 'marty visit'.");
        return;
    }
    for entry in trace.entries.iter().rev().take(last) {
        match entry {
            Signal::Visit { path, ts } => info!(
                "{} [{}] {}",
                "🚶".truecolor(0, 255, 0),
                ts.to_string().truecolor(192, 192, 192),
                format_path_with_emojis(path).bright_white()
            ),
            Signal::Reinforce { path, weight, ts } => info!(
                "{} [{}] {} (Weight: {})",
                "💪".truecolor(255, 255, 0),
                ts.to_string().truecolor(192, 192, 192),
                format_path_with_emojis(path).bright_white(),
                weight.to_string().bold()
            ),
            _ => {}
        }
    }
}

