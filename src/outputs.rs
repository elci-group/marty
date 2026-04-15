use colored::*;
use crate::memory::{Hotspots, Beliefs, Trace};
use crate::signals::Signal;

use std::env;
use comfy_table::{Table, Cell, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};

pub fn success(msg: &str) {
    println!("✅ {}", msg.green().bold());
}

pub fn error(msg: &str) {
    eprintln!("❌ {}", msg.red().bold());
}

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

pub fn format_relative_time(ts: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - ts;
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{} mins ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else {
        format!("{} days ago", diff / 86400)
    }
}

pub fn print_visit(path: &str) {
    println!("📍 Visited: {}", format_path_with_emojis(path).cyan());
}

pub fn print_hotspots(hotspots: &Hotspots, top: usize) {
    println!("🔥 Top Hotspots");
    let items = hotspots.items.lock();
    let mut sorted_hotspots: Vec<_> = items.values().collect();
    sorted_hotspots.sort_by(|a, b| b.energy.partial_cmp(&a.energy).unwrap_or(std::cmp::Ordering::Equal));

    if sorted_hotspots.is_empty() {
        println!("No hotspots recorded yet. Use 'marty visit <path>' to start.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Rank", "Path", "Energy", "Score"]);

    for (i, hs) in sorted_hotspots.iter().take(top).enumerate() {
        let rank = match i {
            0 => "🥇".to_string(),
            1 => "🥈".to_string(),
            2 => "🥉".to_string(),
            _ => format!("#{}", i + 1),
        };
        table.add_row(vec![
            Cell::new(rank),
            Cell::new(format_path_with_emojis(&hs.path)).fg(comfy_table::Color::White),
            Cell::new("⚡".repeat((hs.energy / 10.0).ceil() as usize)).fg(comfy_table::Color::Yellow),
            Cell::new(hs.energy.to_string()).fg(comfy_table::Color::DarkGrey),
        ]);
    }
    println!("{table}");
}

pub fn print_beliefs(beliefs: &Beliefs) {
    println!("🧠 Beliefs Network");
    if beliefs.nodes.is_empty() {
        println!("No beliefs formed yet. Try visiting a few directories.");
        return;
    }
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Label", "Path", "Metadata"]);

    for (id, node) in &beliefs.nodes {
        let mut meta_str = String::new();
        for (key, value) in &node.metadata {
            meta_str.push_str(&format!("{}: {}\n", key, value));
        }
        table.add_row(vec![
            Cell::new(format!("🔹 {}", node.label)).fg(comfy_table::Color::Blue),
            Cell::new(format_path_with_emojis(&id.to_string())).fg(comfy_table::Color::DarkGrey),
            Cell::new(meta_str.trim()),
        ]);
    }
    println!("{table}");
}

pub fn print_trace(trace: &Trace, last: usize) {
    println!("📜 Recent Activity Trace");
    if trace.entries.is_empty() {
        println!("No activity recorded yet. Get started with 'marty visit'.");
        return;
    }
    
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Type", "Timestamp", "Path", "Details"]);

    for entry in trace.entries.iter().rev().take(last) {
        match entry {
            Signal::Visit { path, ts } => {
                table.add_row(vec![
                    Cell::new("🚶").fg(comfy_table::Color::Green),
                    Cell::new(format_relative_time(*ts)).fg(comfy_table::Color::DarkGrey),
                    Cell::new(format_path_with_emojis(path)).fg(comfy_table::Color::White),
                    Cell::new(""),
                ]);
            },
            Signal::Reinforce { path, weight, ts } => {
                table.add_row(vec![
                    Cell::new("💪").fg(comfy_table::Color::Yellow),
                    Cell::new(format_relative_time(*ts)).fg(comfy_table::Color::DarkGrey),
                    Cell::new(format_path_with_emojis(path)).fg(comfy_table::Color::White),
                    Cell::new(format!("Weight: {}", weight)).fg(comfy_table::Color::DarkYellow),
                ]);
            },
            Signal::Tag { path, tag, ts } => {
                table.add_row(vec![
                    Cell::new("🏷️").fg(comfy_table::Color::Cyan),
                    Cell::new(format_relative_time(*ts)).fg(comfy_table::Color::DarkGrey),
                    Cell::new(format_path_with_emojis(path)).fg(comfy_table::Color::White),
                    Cell::new(format!("Tag: {}", tag)).fg(comfy_table::Color::Magenta),
                ]);
            },
            _ => {}
        }
    }
    println!("{table}");
}

