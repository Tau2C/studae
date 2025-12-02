# GEMINI.MD: AI Collaboration Guide

This document provides essential context for AI models interacting with this project. Adhering to these guidelines will ensure consistency and maintain code quality.

## 1. Project Overview & Purpose

+ **Primary Goal:** This is a cross-platform mobile application, "Studae," built with Flutter and Rust. It serves as a modern client for the USOS (Uniwersytecki System Obsługi Studiów) university management system common in Poland. The project includes a Rust-based proxy server (`usos_proxy_server`) that translates modern OAuth2 authentication from the app into the legacy OAuth1a required by the USOS API.
+ **Business Domain:** Education / Higher Education Software.
+ **Documentation:** All project documentation is located in the `/docs` directory. The main entry point is `/docs/README.md`, which contains an index and rules for AI-driven documentation maintenance.

## 2. Core Technologies & Stack

+ **Languages:**
  - Dart
  - Rust (2024 Edition)
+ **Frameworks & Runtimes:**
  - **Frontend:** Flutter
  - **Backend/Proxy:** Rust, with `axum` as the web framework and `tokio` as the async runtime.
  - **Integration:** `flutter_rust_bridge` is used to link the Flutter frontend with the Rust backend logic.
+ **Databases:** No database is used. The authentication proxy stores session information in memory during the login flow.
+ **Key Libraries/Dependencies:**
  - **Flutter:** `flutter_rust_bridge`
  - **Rust:** `axum`, `reqwest`, `oauth2`, `reqwest-oauth1`, `serde`, `tokio`, `flutter_rust_bridge_codegen`, `thiserror`.
+ **Package Manager(s):**
  - `pub` for Dart/Flutter.
  - `cargo` for Rust.

## 3. Coding Conventions & Style Guide

+ **Formatting:**
  - **Dart:** Follows the rules defined in `package:flutter_lints/flutter.yaml`, which uses the standard `dart format` (2-space indentation).
  - **Rust:** Assumed to follow standard `rustfmt` conventions.
+ **Naming Conventions:**
  - **Dart:** `camelCase` for variables and functions; `PascalCase` for classes.
  - **Rust:** `snake_case` for variables, functions, and file names; `PascalCase` for structs and enums.
+ **API Design:** The `usos_proxy_server` provides an internal REST-like API for the Flutter app, exposing `/authorize`, `/callback`, and `/token` endpoints to facilitate an OAuth2 PKCE flow.
+ **Error Handling:**
  - **Rust:** The proxy server uses `anyhow` for general error handling and `tracing` for logging. The `usos_lib` crate uses `thiserror` for custom error types.
  - **Dart:** Standard `Future`-based error handling with `async`/`await` and `try-catch` blocks.

## 4. Documentation

The `/docs` directory is managed by an AI agent and serves as the project's living memory.
- **Entry Point:** Always start with `/docs/README.md`. It contains the index and rules for documentation.
- **AI Instructions:** Specific instructions for AI agent interaction are in `/docs/AGENT_INSTRUCTIONS.md`.
- **Maintenance:** The AI is responsible for keeping the documentation index and content up-to-date, refactoring aggressively, and creating new files only when necessary.

## 5. Development & Testing Workflow

+ **Local Development Environment:** The project uses `direnv` and Nix (via `default.nix`) to create a consistent and reproducible development environment. To start the app, use `flutter run`. To run the proxy server, use `cargo run -p usos_proxy_server`.
+ **Testing:**
  - **Flutter:** Run unit and widget tests with `flutter test`. Integration tests are located in `app/integration_test/`.
  - **Rust:** Run tests for Rust crates with `cargo test`.
+ **CI/CD Process:** There is currently no CI/CD pipeline configured for automated testing or deployment in the `.github/workflows` directory.

## 6. Specific Instructions for AI Collaboration

+ **Working with documentation:** Read `/docs/AGENT_INSTRUCTIONS.md` and follow the rules and prompts within it for interacting with documentation.
+ **Working with code:** Always update the index in `/docs/README.md` after creating, deleting or changing any code.
+ **Contribution Guidelines:** No `CONTRIBUTING.md` file exists. Follow existing code patterns and conventions.
+ **Infrastructure (IaC):** There is no Infrastructure as Code directory in this project.
+ **Security:** Be mindful of security. The `usos_proxy_server` contains hardcoded placeholder secrets. For production, these must be loaded securely from environment variables, not committed to the repository.
+ **Dependencies:**
  - To add a Dart/Flutter dependency, use `flutter pub add <package>`.
  - To add a Rust dependency, use `cargo add <crate>`.
+ **Commit Messages:** Follow the Conventional Commits specification (e.g., `feat:`, `fix:`, `docs:`, `refactor(scope):`). Ensure messages have a concise subject line and a clear, descriptive body explaining the 'what' and 'why' of the change.
