use std::env;
use colored::*;
use std::io::{stdin, stdout, Write};
use clap::Parser;
use std::sync::Arc;
use parking_lot::Mutex;

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

use std::str::FromStr;

use crate::cli::{Cli, Commands};
use crate::memory::{Hotspots, Beliefs, Trace};
use crate::error::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use crate::config::SETTINGS;

fn main() -> Result<()> {
    let log_level = SETTINGS.read().unwrap().log_level.clone();
    let level = Level::from_str(&log_level).unwrap_or(Level::INFO);
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    metrics::register_metrics();

    info!("🚀 Marty is soaring into action!");

    let hotspots = Arc::new(Mutex::new(Hotspots::default()));
    let beliefs = Arc::new(Mutex::new(Beliefs::default()));
    let trace = Arc::new(Mutex::new(Trace::default()));



    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Visit { path } => {
                outputs::print_visit(&path);
            }
            Commands::Hotspots { top } => {
                outputs::print_hotspots(&hotspots.lock(), top);
            }
            Commands::Beliefs => {
                outputs::print_beliefs(&beliefs.lock());
            }
            Commands::Trace { last } => {
                outputs::print_trace(&trace.lock(), last);
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

    info!("🚀 Marty is now in interactive mode...");
    info!("Use symbols like < (back), > (forward), ^ (up), or a {{subdir}} to navigate.");
    info!("Type 'exit' or press Ctrl+C to leave.");

    loop {
        print!("{}", "» ".truecolor(0, 191, 255).bold());
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            outputs::success("👋 Goodbye from Marty! See you next time.");
            break;
        }

        if input.is_empty() {
            continue;
        }

        match marty::traverse(input, &mut history) {
            Ok(new_dir) => {
                env::set_current_dir(&new_dir)?;
            }
            Err(e) => outputs::error(&e.to_string()),
        }
    }
    Ok(())
}
