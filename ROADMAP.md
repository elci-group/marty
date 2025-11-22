# Marty Enterprise-Readiness Roadmap

This document outlines the path to evolving Marty from a personal tool to an enterprise-grade application. The roadmap is divided into three phases, each focusing on a different aspect of production readiness.

## Phase 1: Foundational Stability

This phase focuses on core reliability, testability, and maintainability. These are the essential first steps for any production system.

*   **1. Comprehensive Testing:**
    *   **Unit Tests:** Add unit tests for core business logic in `marty.rs` and `model.rs`.
    *   **Integration Tests:** Create integration tests for the CLI (`cli.rs`) and the HTTP server (`http.rs`).
    *   **Goal:** Achieve >80% code coverage.

*   **2. Structured Logging:**
    *   Integrate the `tracing` crate for structured, level-based logging.
    *   Add logs for critical events, errors, and key application lifecycle events.
    *   Allow log level configuration via an environment variable.

*   **3. External Configuration:**
    *   Introduce a configuration file (e.g., `marty.toml`) for settings like the server port, database path, and logging levels.
    *   Use a library like `config` or `serde` to parse the configuration file.
    *   Support environment variables as overrides for configuration values.

*   **4. Robust Error Handling:**
    *   Integrate a crate like `thiserror` to create specific, typed errors.
    *   Replace all instances of `.unwrap()` and `.expect()` with proper error handling and propagation.

## Phase 2: Hardening and Automation

This phase is about making the application more robust, secure, and easier to manage.

*   **1. Continuous Integration & Deployment (CI/CD):**
    *   Set up a CI pipeline (e.g., using GitHub Actions) that runs `cargo fmt`, `cargo clippy`, and `cargo test` on every commit.
    *   Create a CD pipeline to automatically build and publish releases when a new tag is pushed.

*   **2. API Versioning and Documentation:**
    *   Version the HTTP API (e.g., `/api/v1/...`).
    *   Generate API documentation using a tool like OpenAPI/Swagger.
    *   Document the public Rust API with `cargo doc`.

*   **3. Security Audit:**
    *   Run `cargo audit` to check for dependencies with known vulnerabilities.
    *   Review all code handling user input (CLI and HTTP) for potential security issues (e.g., path traversal).

*   **4. User Documentation:**
    *   Create a `docs` directory with user-facing documentation on how to install, configure, and use Marty.
    *   Add a `CONTRIBUTING.md` file for developers.

## Phase 3: Scaling & Observability

This phase prepares Marty for larger-scale deployments and easier operational management.

*   **1. Metrics and Monitoring:**
    *   Expose application metrics (e.g., API request latency, error rates) via a `/metrics` endpoint in a Prometheus-compatible format.
    *   Integrate a crate like `prometheus` or `metrics`.

*   **2. Refactoring for Extensibility:**
    *   Refactor the core logic in `marty.rs` to be more modular, allowing for different storage backends or scoring algorithms in the future.

*   **3. Cross-Platform Support:**
    *   Test and ensure full functionality on Windows, macOS, and Linux.
    *   Provide pre-compiled binaries for each platform as part of the release process.

*   **4. Containerization:**
    *   Create a `Dockerfile` to build and run Marty in a containerized environment.
    *   Publish the container image to a registry (e.g., Docker Hub, GHCR).
