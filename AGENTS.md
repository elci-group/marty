# Marty Agent Guide

This document provides guidance for AI agents working in the Marty codebase.

## Project Overview

Marty is a Rust-based command-line tool for intelligent directory navigation. It tracks "hotspots" (frequently visited directories), "beliefs" (directory relationships), and "traces" (recent navigation history). It also includes a `warp`-based HTTP server for a dashboard view.

## Commands

This is a standard Rust project using Cargo.

*   **Build:** `cargo build`
*   **Run:** `cargo run`
*   **Test:** `cargo test`
*   **Check:** `cargo check` (fast compilation check)
*   **Format:** `cargo fmt`
*   **Lint:** `cargo clippy`

## Code Organization

*   `src/main.rs`: Main application entry point. Contains the interactive directory navigation loop.
*   `src/cli.rs`: Defines the command-line interface using `clap`. Subcommands include `visit`, `hotspots`, `beliefs`, and `trace`.
*   `src/http.rs`: Implements a `warp` HTTP server to expose hotspots, beliefs, and traces via a JSON API.
*   `src/marty.rs`: Likely contains the core logic for managing hotspots, beliefs, and traces.
*   `src/model.rs`: Defines the data structures for the application.
*   `src/scheduler.rs`: Potentially manages background tasks or updates.
*   `src/signals.rs`: May handle process signals.
*   `src/outputs.rs`: Handles formatted output to the console.

## Code Style and Conventions

*   The codebase uses standard Rust conventions and formatting, enforced by `cargo fmt`.
*   The `colored` crate is used for terminal output. Maintain this pattern for user-facing messages.
*   Dependencies are managed in `Cargo.toml`.

## Testing

There are no apparent tests in the codebase. When adding new functionality, please add corresponding unit or integration tests using the standard Rust testing framework (`#[cfg(test)]`). Run tests with `cargo test`.
