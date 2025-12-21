# 🚀 Marty: Your Intelligent File System Navigator 🧭

Welcome to **Marty**, the smart way to get around your filesystem!

Marty is a command-line companion that learns your habits to make directory navigation faster and more intuitive. Spend less time `cd`-ing and more time working.

## ✨ Features

*   **🔥 Hotspots:** Tracks your most frequently visited directories and assigns them "energy" based on usage.
*   **🧠 Beliefs:** Learns relationships between directories to predict where you might want to go next.
*   **📜 Trace:** Keeps a searchable history of your recent navigation path.
*   **🤖 Interactive Mode:** A powerful REPL for lightning-fast jumps and exploration.
*   **🌐 HTTP Dashboard:** A built-in web UI to visualize your navigation patterns and stats.

## 🛠️ Installation

### Prerequisites

*   **Rust**: You need the Rust toolchain installed (Cargo). [Get Rust](https://www.rust-lang.org/tools/install).

### Build from Source

Clone the repository and build using Cargo:

```bash
git clone git@github.com:elci-group/marty.git
cd marty
cargo build --release
```

The binary will be available at `./target/release/marty`.

## 📖 Usage

### Basic Commands

Marty can be used directly from the command line with subcommands:

#### 📝 Visit a Directory
Reinforce a path as a hotspot. This is typically hooked into your shell's `cd` command.

```bash
marty visit /path/to/project
```

#### 🔥 View Hotspots
See your most frequent destinations, ranked by energy.

```bash
# Show top 5 hotspots (default)
marty hotspots

# Show top 10
marty hotspots --top 10
```

#### 🧠 View Beliefs
Show the learned relationships between directories.

```bash
marty beliefs
```

#### 📜 View Trace
See your recent navigation history.

```bash
# Show last 10 entries (default)
marty trace

# Show last 20
marty trace --last 20
```

### 🤖 Interactive Mode

Run `marty` without arguments to enter the interactive mode (if configured):

```bash
marty
```

### 🌐 HTTP Dashboard

Marty runs a local HTTP server to visualize your data. By default, it runs on port **7777**.

Access it at: `http://localhost:7777`

## ⚙️ Configuration

Configuration is handled via the `Marty.toml` file.

```toml
# Marty Configuration

# Port for the HTTP server
server_port = 7777

# Log level ("trace", "debug", "info", "warn", "error")
log_level = "info"
```

## 🤝 Contributing

We welcome contributions! Please check [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to get started.

## 🗺️ Roadmap

See [ROADMAP.md](ROADMAP.md) for planned features and future direction.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
