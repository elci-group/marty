# 🚀 Marty: A Filesystem Companion, Not a `cd` Replacement 🧭

Marty is a command-line tool that quietly learns which directories you work in and
turns that history into two things: fast recall ("where was I working recently?
where do I usually go for X?") and instant project context ("show me this project's
tree and source in one shot"). It does **not** change your shell's working directory —
see [What Marty is (and isn't)](#-what-marty-is-and-isnt) below before you go looking
for a `cd` alternative here.

## ✨ Features

*   **🔥 Hotspots** — Tracks directories you `visit` and ranks them by "energy": a
    score that goes up each visit and decays 5% per hour, so frequently *and*
    recently used directories rise to the top.
*   **📜 Trace** — A timestamped log of your recent `visit` and `tag` activity.
*   **🏷️ Tags** — Attach freeform labels to directories for later lookup
    (`marty tag ~/work/api backend`).
*   **🔭 Scout** — The most useful command for day-to-day work: detects a project's
    type (Rust, Python, Node, Godot, Flutter, iOS, Android, or a generic fallback),
    then produces a directory tree plus a filtered, token-capped source snapshot in a
    single call. Under the hood it wraps `bart` (tree) and `bound` (source bundling)
    so you don't have to pick the right filters yourself.
*   **🧠 Beliefs** — A lightweight key-value store of per-directory metadata (tags,
    relationships you add manually). This is intentionally simple today; it does not
    yet infer relationships automatically.
*   **🖥️ Interactive REPL** — Run `marty` with no arguments for a small navigation
    shell with vim-style motions (`<` back, `>` forward, `^` up a level, `{subdir}` to
    jump into a child directory), back/forward history, and a breadcrumb trail. This
    *does* change directory — but only inside that REPL session, not your regular shell.
*   **📺 TUI** — `marty tui` opens a tabbed `ratatui` dashboard over your hotspots,
    beliefs, and trace, navigable with `j`/`k`/arrows/Tab.
*   **🌐 HTTP endpoint** — `marty server` exposes `/health` and Prometheus-format
    `/metrics` on port 7777 for monitoring. (The `/api/v1/hotspots`, `/api/v1/beliefs`,
    and `/api/v1/trace` JSON endpoints are present in the code but currently return
    404 — a known bug, not yet a usable dashboard API.)

## 🧭 What Marty is (and isn't)

Marty's own `--help` text used to say "spend less time `cd`-ing," which oversold it.
None of `visit`, `hotspots`, `beliefs`, `trace`, `scout`, `tag`, or `server` change
your shell's current directory — a subprocess like Marty structurally can't do that
without a sourced shell function, and Marty doesn't ship one. Marty's job is to
**remember and summarize**, not to move you around. Pair it with your normal `cd`:
use `marty hotspots`/`marty trace` to decide *where* to go, then `cd` there yourself
(optionally following up with `marty visit <path>` to reinforce it as a hotspot).

The one exception is the interactive REPL (`marty`, no subcommand) — it has its own
real navigation model with history and vim-style motions, but it's a separate shell
session, not something your existing terminal gains for free.

## 🛠️ Installation

### Prerequisites

*   **Rust** (Cargo) to build Marty itself. [Get Rust](https://www.rust-lang.org/tools/install).
*   **`bart` and `bound`** on your `PATH` if you want to use `marty scout` — it
    shells out to both. Every other command works without them.

### Build from Source

```bash
git clone git@github.com:elci-group/marty.git
cd marty
cargo build --release
```

The binary will be available at `./target/release/marty` (release build, ~8.5 MB).

## 📖 Usage

### 📝 Visit a Directory

Reinforce a path as a hotspot. Call this yourself after navigating somewhere worth
remembering — Marty doesn't hook your shell's `cd` automatically.

```bash
marty visit /path/to/project
```

### 🔥 View Hotspots

```bash
marty hotspots            # top 5, ranked by energy
marty hotspots --top 10   # top 10
marty hotspots --json     # for scripting/agents
```

### 🔭 Scout a Project

Get a tree plus filtered source snapshot of a directory in one call:

```bash
marty scout ~/work/my-api
marty scout ~/work/my-api --json --depth 3 --token-limit 8000
```

`project_type` is auto-detected from marker files (`Cargo.toml`, `package.json`,
`pyproject.toml`, `project.godot`, etc.) and used to pick which file extensions get
bundled into the snapshot. `--token-limit` caps tokens **per file**, not in total.

### 🧠 View Beliefs / 🏷️ Tag a Directory

```bash
marty beliefs
marty tag /path/to/project rust-backend
```

### 📜 View Trace

```bash
marty trace            # last 10 entries
marty trace --last 20  # last 20
marty trace --json
```

### 🤖 Interactive REPL

```bash
marty
```

Navigate with `<` (back), `>` (forward), `^` (up), or `{subdir}` (jump into a child
directory). Type `exit` or press Ctrl+C to leave.

### 📺 TUI

```bash
marty tui
```

### 🌐 HTTP Server

```bash
marty server
```

Serves `http://127.0.0.1:7777/health` and `/metrics` (Prometheus format). The
`/api/v1/*` JSON endpoints exist in the code but are currently broken (404) — don't
build tooling against them yet.

## ⚡ Performance

Numbers below are from real runs on this machine (release build, `~/.marty/state.json`
holding real usage history — a few KB, well under the caps described below):

| Command | Typical time | Notes |
|---|---|---|
| `hotspots` / `trace` / `beliefs` | 2–6 ms | Dominated by process startup; state reads are effectively free at this scale. |
| `scout` on a ~10-file Rust project | ~10 ms | |
| `scout` on a ~30-file Rust project (200 KB of source) | ~100 ms | Nearly all of this is the `bart` (~40 ms) and `bound` (~90 ms) subprocesses it wraps — Marty's own detection/glue logic adds negligible overhead. |

State stays fast indefinitely because it's capped on write: 100 hotspots, 500 trace
entries, 1000 beliefs (oldest/lowest-energy entries are dropped first). In months of
real usage on this machine, `state.json` is under 4 KB.

## ⚙️ Configuration

Configuration is handled via `Marty.toml` in the current directory, with `MARTY_*`
environment variables as overrides (e.g. `MARTY_SERVER_PORT=8080`):

```toml
# Marty Configuration

# Port for the HTTP server
server_port = 7777

# Log level ("trace", "debug", "info", "warn", "error")
log_level = "info"
```

State (hotspots, beliefs, trace) persists to `~/.marty/state.json` by default,
written atomically with `0600` permissions since it can contain sensitive paths.
Override the location with `-s/--state <PATH>`. Logs go to `~/.marty/marty.log`.

## 🤝 Contributing

We welcome contributions! Please check [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to get started.

## 🗺️ Roadmap

See [ROADMAP.md](ROADMAP.md) for planned features and future direction.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
