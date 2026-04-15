use std::env;
use colored::*;
use clap::Parser;
use std::sync::Arc;
use parking_lot::Mutex;
use std::str::FromStr;
use tokio::runtime::Runtime;
use std::time::Duration;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

mod marty;
mod cli;
mod http;
mod memory;
mod model;
mod outputs;
mod scheduler;
mod signals;
mod error;
mod config;
mod metrics;
mod tui;

use crate::cli::{Cli, Commands};
use crate::memory::{Hotspots, Beliefs, Trace, Hotspot};
use crate::error::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use crate::config::SETTINGS;
use crate::signals::Signal;

fn main() -> Result<()> {
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
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    metrics::register_metrics();

    info!("🚀 Marty is soaring into action!");

    let hotspots = Arc::new(Mutex::new(Hotspots::default()));
    let beliefs = Arc::new(Mutex::new(Beliefs::default()));
    let trace = Arc::new(Mutex::new(Trace::default()));

    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    let h_hotspots = Arc::clone(&hotspots);
    let h_beliefs = Arc::clone(&beliefs);
    let h_trace = Arc::clone(&trace);
    
    rt.spawn(async move {
        http::run_server(h_hotspots, h_beliefs, h_trace).await;
    });

    let decay_interval = Duration::from_secs(3600); // 1 hour
    let scheduler = scheduler::DecayScheduler::new(decay_interval);
    let s_hotspots = Arc::clone(&hotspots);
    scheduler.start(move || {
        let hs = s_hotspots.lock();
        let mut items = hs.items.lock();
        for hotspot in items.values_mut() {
            hotspot.energy *= 0.95; // 5% decay
        }
    });

    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Visit { path } => {
                let abs_path = if let Ok(p) = std::fs::canonicalize(&path) {
                    p.to_string_lossy().to_string()
                } else {
                    path.clone()
                };
                
                let hs = hotspots.lock();
                let mut items = hs.items.lock();
                let entry = items.entry(abs_path.clone()).or_insert(Hotspot {
                    path: abs_path.clone(),
                    energy: 0.0,
                    last_ts: Signal::timestamp_now(),
                });
                entry.energy += 1.0;
                entry.last_ts = Signal::timestamp_now();
                
                trace.lock().entries.push(Signal::Visit {
                    path: abs_path,
                    ts: Signal::timestamp_now(),
                });
                
                outputs::print_visit(&path);
            }
            Commands::Hotspots { top } => {
                outputs::print_hotspots(&hotspots.lock(), top);
            }
            Commands::Beliefs => {
                outputs::print_beliefs(&beliefs.lock());
            }
            Commands::Tag { path, tag } => {
                let abs_path = if let Ok(p) = std::fs::canonicalize(&path) {
                    p.to_string_lossy().to_string()
                } else {
                    path.clone()
                };
                
                trace.lock().entries.push(Signal::Tag {
                    path: abs_path.clone(),
                    tag: tag.clone(),
                    ts: Signal::timestamp_now(),
                });
                
                outputs::success(&format!("Tagged {} with '{}'", path, tag));
            }
            Commands::Trace { last } => {
                outputs::print_trace(&trace.lock(), last);
            }
            Commands::Tui => {
                let app = tui::TuiApp::new(hotspots, beliefs, trace);
                app.run()?;
            }
        }
    } else {
        run_interactive_loop()?;
    }
    Ok(())
}

fn run_interactive_loop() -> Result<()> {
    let mut history = marty::History::default();
    let current_dir = env::current_dir()?;
    history.backward.push(current_dir);

    println!("🚀 Marty is now in interactive mode...");
    println!("Use symbols like < (back), > (forward), ^ (up), or a {{subdir}} to navigate.");
    println!("Type 'exit' or press Ctrl+C to leave.");

    let mut rl = DefaultEditor::new().map_err(|e| crate::error::MartyError::Config(e.to_string()))?;

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
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                outputs::success("👋 Goodbye from Marty! See you next time.");
                break;
            },
            Err(err) => {
                outputs::error(&format!("Error: {:?}", err));
                break;
            }
        }
    }
    Ok(())
}
