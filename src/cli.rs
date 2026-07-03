use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
}

#[derive(Parser, Debug)]
#[command(
    author = "Rory Spring",
    version = "0.1",
    about = "🚀 Marty: Your Intelligent File System Navigator 🧭",
    long_about = "Marty is a command-line companion that learns your habits to make directory navigation faster and more intuitive. Spend less time `cd`-ing and more time working.",
    styles = get_styles()
)]
pub struct Cli {
    /// Path to the persisted state file (default: ~/.marty/state.json)
    #[arg(short, long, value_name = "PATH", global = true)]
    pub state: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 📝 Visit a directory to reinforce it as a hotspot
    Visit {
        /// The directory path to visit
        #[arg(value_name = "PATH")]
        path: String,
    },
    /// 🔥 List the top N hotspots by energy
    Hotspots {
        /// The number of hotspots to show
        #[arg(short, long, default_value_t = 5)]
        top: usize,
        /// Output raw JSON instead of the formatted table
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// 🧠 Show all directory beliefs (relationships)
    Beliefs {
        /// Output raw JSON instead of the formatted table
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// 🏷️ Tag a directory
    Tag {
        /// The directory path to tag
        #[arg(value_name = "PATH")]
        path: String,
        /// The tag to assign
        #[arg(value_name = "TAG")]
        tag: String,
    },
    /// 📜 Show the last N trace (navigation history) entries
    Trace {
        /// The number of trace entries to show
        #[arg(short, long, default_value_t = 10)]
        last: usize,
        /// Output raw JSON instead of the formatted table
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// 🖥️ Open the interactive TUI
    Tui,
    /// 🌐 Start the HTTP dashboard server
    Server,
    /// 🔭 Scout a directory: tree + source snapshot
    Scout {
        /// The directory path to scout
        #[arg(value_name = "PATH")]
        path: String,
        /// Output raw JSON instead of pretty-printed text
        #[arg(long, default_value_t = false)]
        json: bool,
        /// Maximum recursion depth for the directory tree
        #[arg(short, long, default_value_t = 3)]
        depth: usize,
        /// Max tokens per file for the bundled snapshot
        #[arg(short, long, default_value_t = 8000)]
        token_limit: usize,
    },
}
