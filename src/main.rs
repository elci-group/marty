use clap::Parser;
use colored::*;
use parking_lot::Mutex;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod cli;
mod config;
mod error;
mod http;
mod marty;
mod memory;
mod metrics;
mod model;
mod outputs;
mod persistence;
mod scheduler;
mod signals;
mod tui;

use crate::cli::{Cli, Commands};
use crate::config::SETTINGS;
use crate::error::Result;
use crate::memory::{Beliefs, Hotspot, Hotspots, Trace};
use crate::persistence::default_state_path;
use crate::signals::Signal;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_logging();

    let state_path: PathBuf = cli
        .state
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(default_state_path);

    let hotspots = Arc::new(Mutex::new(Hotspots::default()));
    let beliefs = Arc::new(Mutex::new(Beliefs::default()));
    let trace = Arc::new(Mutex::new(Trace::default()));

    persistence::load(&state_path, &hotspots, &beliefs, &trace);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    match cli.command {
        Some(Commands::Server) => {
            metrics::register_metrics();
            let h_hotspots = Arc::clone(&hotspots);
            let h_beliefs = Arc::clone(&beliefs);
            let h_trace = Arc::clone(&trace);
            rt.spawn(async move {
                http::run_server(h_hotspots, h_beliefs, h_trace).await;
            });
            start_decay_scheduler(Arc::clone(&hotspots));
            start_periodic_save(
                state_path.clone(),
                Arc::clone(&hotspots),
                Arc::clone(&beliefs),
                Arc::clone(&trace),
                &rt,
            );
            rt.block_on(shutdown_with_save(
                state_path.clone(),
                Arc::clone(&hotspots),
                Arc::clone(&beliefs),
                Arc::clone(&trace),
            ));
        }
        Some(Commands::Tui) => {
            metrics::register_metrics();
            let h_hotspots = Arc::clone(&hotspots);
            let h_beliefs = Arc::clone(&beliefs);
            let h_trace = Arc::clone(&trace);
            rt.spawn(async move {
                http::run_server(h_hotspots, h_beliefs, h_trace).await;
            });
            start_decay_scheduler(Arc::clone(&hotspots));
            start_periodic_save(
                state_path.clone(),
                Arc::clone(&hotspots),
                Arc::clone(&beliefs),
                Arc::clone(&trace),
                &rt,
            );
            let app = tui::TuiApp::new(hotspots, beliefs, trace);
            app.run()?;
        }
        Some(Commands::Scout {
            path,
            json,
            depth,
            token_limit,
        }) => {
            run_scout(&path, depth, token_limit, json)?;
        }
        Some(Commands::Hotspots { top, json }) => {
            let hs = hotspots.lock();
            if json {
                print_hotspots_json(&hs);
            } else {
                outputs::print_hotspots(&hs, top);
            }
        }
        Some(Commands::Visit { path }) => {
            let abs_path = canonicalize_or_keep(&path);
            {
                let hs = hotspots.lock();
                let mut items = hs.items.lock();
                let entry = items.entry(abs_path.clone()).or_insert(Hotspot {
                    path: abs_path.clone(),
                    energy: 0.0,
                    last_ts: Signal::timestamp_now(),
                });
                entry.energy += 1.0;
                entry.last_ts = Signal::timestamp_now();
            }
            {
                let mut tr = trace.lock();
                tr.entries.push(Signal::Visit {
                    path: abs_path,
                    ts: Signal::timestamp_now(),
                });
            }
            outputs::print_visit(&path);
            if let Err(e) = persistence::save(&state_path, &hotspots, &beliefs, &trace) {
                warn!("failed to save state: {}", e);
            }
        }
        Some(Commands::Beliefs { json }) => {
            let b = beliefs.lock();
            if json {
                print_beliefs_json(&b);
            } else {
                outputs::print_beliefs(&b);
            }
        }
        Some(Commands::Tag { path, tag }) => {
            let abs_path = canonicalize_or_keep(&path);
            let mut tr = trace.lock();
            tr.entries.push(Signal::Tag {
                path: abs_path.clone(),
                tag: tag.clone(),
                ts: Signal::timestamp_now(),
            });
            outputs::success(&format!("Tagged {} with '{}'", path, tag));
            if let Err(e) = persistence::save(&state_path, &hotspots, &beliefs, &trace) {
                warn!("failed to save state: {}", e);
            }
        }
        Some(Commands::Trace { last, json }) => {
            let t = trace.lock();
            if json {
                print_trace_json(&t, last);
            } else {
                outputs::print_trace(&t, last);
            }
        }
        None => {
            start_decay_scheduler(Arc::clone(&hotspots));
            run_interactive_loop()?;
        }
    }

    Ok(())
}

fn setup_logging() {
    let log_level = SETTINGS.read().unwrap().log_level.clone();
    let level = Level::from_str(&log_level).unwrap_or(Level::INFO);

    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let log_dir = home_dir.join(".marty");
    std::fs::create_dir_all(&log_dir).unwrap_or_default();
    let file_appender = tracing_appender::rolling::never(log_dir, "marty.log");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_writer(file_appender)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("🚀 Marty is soaring into action!");
}

fn start_decay_scheduler(hotspots: Arc<Mutex<Hotspots>>) {
    let decay_interval = Duration::from_secs(3600); // 1 hour
    let scheduler = scheduler::DecayScheduler::new(decay_interval);
    scheduler.start(move || {
        let hs = hotspots.lock();
        let mut items = hs.items.lock();
        for hotspot in items.values_mut() {
            hotspot.energy *= 0.95; // 5% decay
        }
    });
}

fn start_periodic_save(
    state_path: PathBuf,
    hotspots: Arc<Mutex<Hotspots>>,
    beliefs: Arc<Mutex<Beliefs>>,
    trace: Arc<Mutex<Trace>>,
    rt: &tokio::runtime::Runtime,
) {
    rt.spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let path = state_path.clone();
            let hs = Arc::clone(&hotspots);
            let bl = Arc::clone(&beliefs);
            let tr = Arc::clone(&trace);
            let _ = tokio::task::spawn_blocking(move || {
                if let Err(e) = persistence::save(&path, &hs, &bl, &tr) {
                    warn!("failed to save state: {}", e);
                }
            })
            .await;
        }
    });
}

async fn shutdown_with_save(
    state_path: PathBuf,
    hotspots: Arc<Mutex<Hotspots>>,
    beliefs: Arc<Mutex<Beliefs>>,
    trace: Arc<Mutex<Trace>>,
) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut terminate = signal(SignalKind::terminate()).ok();
        let mut interrupt = signal(SignalKind::interrupt()).ok();
        tokio::select! {
            _ = async { terminate.as_mut()?.recv().await }, if terminate.is_some() => {}
            _ = async { interrupt.as_mut()?.recv().await }, if interrupt.is_some() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }

    let _ = tokio::task::spawn_blocking(move || {
        if let Err(e) = persistence::save(&state_path, &hotspots, &beliefs, &trace) {
            warn!("failed to save state on shutdown: {}", e);
        }
    })
    .await;
}

fn canonicalize_or_keep(path: &str) -> String {
    let expanded = expand_tilde(path);
    std::fs::canonicalize(&expanded)
        .unwrap_or(expanded)
        .to_string_lossy()
        .to_string()
}

fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest);
    }
    PathBuf::from(path)
}

/// Scout a directory: run bart for the tree and bound for a source snapshot.
fn run_scout(path: &str, depth: usize, token_limit: usize, json: bool) -> Result<()> {
    let root = expand_tilde(path);
    if !root.exists() {
        return Err(crate::error::MartyError::Internal(format!(
            "scout target does not exist: {}",
            root.display()
        )));
    }
    if !root.is_dir() {
        return Err(crate::error::MartyError::Internal(format!(
            "scout target is not a directory: {}",
            root.display()
        )));
    }

    let tree = run_bart(&root, depth).map_err(|e| crate::error::MartyError::Internal(e))?;
    let project_type = detect_project_type(&root);
    let filters = bound_filters_for(project_type);
    let mut snapshot = String::new();
    for filter in filters {
        match run_bound(&root, filter, token_limit) {
            Ok(content) if !content.trim().is_empty() => {
                snapshot = content;
                break;
            }
            Ok(_) => continue,
            Err(e) => eprintln!("{} bound filter {} failed: {}", "⚠️".yellow(), filter, e),
        }
    }
    if snapshot.trim().is_empty() {
        snapshot = "(no existing source files to bundle)".to_string();
    }

    if json {
        let output = serde_json::json!({
            "path": root.to_string_lossy().to_string(),
            "project_type": project_type,
            "tree": tree,
            "snapshot": snapshot,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        println!(
            "{} Scouting {} ({})",
            "🔭".cyan(),
            root.display().to_string().yellow(),
            project_type
        );
        println!("\n{}", "📁 Directory tree".bold());
        println!("{}", tree);
        println!("\n{}", "📄 Source snapshot".bold());
        println!("{}", snapshot);
    }

    Ok(())
}

fn run_bart(path: &Path, depth: usize) -> std::result::Result<String, String> {
    let output = Command::new("bart")
        .arg("--json")
        .arg("-d")
        .arg(depth.to_string())
        .arg("-n")
        .arg("0")
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("bart failed to run: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("bart exited with error: {}", stderr));
    }

    let tree = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(tree)
}

fn run_bound(path: &Path, filter: &str, token_limit: usize) -> std::result::Result<String, String> {
    let out_file = std::env::temp_dir().join(format!(
        "marty_bound_{}_{}.txt",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

    let status = Command::new("bound")
        .arg(filter)
        .arg("--meta")
        .arg("--tree")
        .arg("-t")
        .arg(token_limit.to_string())
        .arg("--out")
        .arg(&out_file)
        .arg(path)
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("bound failed to run: {}", e))?;

    if !status.success() {
        let _ = std::fs::remove_file(&out_file);
        return Err("bound exited with error".to_string());
    }

    let content = std::fs::read_to_string(&out_file)
        .map_err(|e| format!("failed to read bound output: {}", e))?;
    let _ = std::fs::remove_file(&out_file);
    Ok(content)
}

fn detect_project_type(root: &Path) -> &'static str {
    if root.join("project.godot").exists() {
        return "godot";
    }
    if root.join("Cargo.toml").exists() {
        return "rust";
    }
    if root.join("pubspec.yaml").exists() {
        return "flutter";
    }
    if root.join("Package.swift").exists() {
        return "ios";
    }
    if root.join("build.gradle").exists()
        || root.join("build.gradle.kts").exists()
        || root.join("settings.gradle").exists()
    {
        return "android";
    }
    if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
        return "python";
    }
    if root.join("package.json").exists() {
        return "vite_web";
    }
    "unknown"
}

fn bound_filters_for(project_type: &str) -> &'static [&'static str] {
    match project_type {
        "godot" => &["[.gd]", "[.tscn]", "[.tres]", "[project.godot]"],
        "rust" => &["[.rs]", "[Cargo.toml]"],
        "vite_web" => &["[.ts]", "[.tsx]", "[.js]", "[.jsx]", "[package.json]"],
        "python" => &["[.py]", "[requirements.txt]", "[pyproject.toml]"],
        "android" => &["[.kt]", "[.java]", "[build.gradle]"],
        "ios" => &["[.swift]", "[.m]", "[Package.swift]"],
        "flutter" => &["[.dart]", "[pubspec.yaml]"],
        _ => &[
            "[.rs]",
            "[.py]",
            "[.ts]",
            "[.tsx]",
            "[.js]",
            "[.jsx]",
            "[.go]",
            "[.swift]",
            "[.kt]",
            "[.java]",
            "[.dart]",
            "[Cargo.toml]",
            "[package.json]",
            "[requirements.txt]",
            "[pyproject.toml]",
        ],
    }
}

fn print_hotspots_json(hotspots: &Hotspots) {
    let items = hotspots.items.lock();
    let mut values: Vec<_> = items.values().collect();
    values.sort_by(|a, b| {
        b.energy
            .partial_cmp(&a.energy)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&values).unwrap_or_else(|_| "[]".to_string())
    );
}

fn print_beliefs_json(beliefs: &Beliefs) {
    println!(
        "{}",
        serde_json::to_string_pretty(&beliefs.nodes).unwrap_or_else(|_| "{}".to_string())
    );
}

fn print_trace_json(trace: &Trace, last: usize) {
    let recent: Vec<_> = trace.entries.iter().rev().take(last).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&recent).unwrap_or_else(|_| "[]".to_string())
    );
}

fn run_interactive_loop() -> Result<()> {
    let mut history = marty::History::default();
    let current_dir = env::current_dir()?;
    history.backward.push(current_dir);

    println!("🚀 Marty is now in interactive mode...");
    println!("Use symbols like < (back), > (forward), ^ (up), or a {{subdir}} to navigate.");
    println!("Type 'exit' or press Ctrl+C to leave.");

    let mut rl =
        DefaultEditor::new().map_err(|e| crate::error::MartyError::Config(e.to_string()))?;

    loop {
        let cur_dir = env::current_dir()?;
        let display_path = outputs::format_path_with_emojis(&cur_dir.to_string_lossy());
        let prompt = format!("{} {} ", display_path.cyan(), "»".green().bold());

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(input);

                if input.eq_ignore_ascii_case("exit") {
                    outputs::success("👋 Goodbye from Marty! See you next time.");
                    break;
                }

                match marty::traverse(input, &mut history) {
                    Ok(new_dir) => {
                        env::set_current_dir(&new_dir)?;
                    }
                    Err(e) => outputs::error(&e.to_string()),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                outputs::success("👋 Goodbye from Marty! See you next time.");
                break;
            }
            Err(err) => {
                outputs::error(&format!("Error: {:?}", err));
                break;
            }
        }
    }
    Ok(())
}
