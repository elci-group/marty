

use clap::{Parser, Subcommand};


#[derive(Parser, Debug)]
#[command(
    author = "Rory Spring",
    version = "0.1",
    about = "🚀 Marty: Your Intelligent File System Navigator 🧭",
    long_about = "Marty is a command-line companion that learns your habits to make directory navigation faster and more intuitive. Spend less time `cd`-ing and more time working."
)]
pub struct Cli {
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
    },
    /// 🧠 Show all directory beliefs (relationships)
    Beliefs,
    /// 📜 Show the last N trace (navigation history) entries
    Trace {
        /// The number of trace entries to show
        #[arg(short, long, default_value_t = 10)]
        last: usize,
    },
}

