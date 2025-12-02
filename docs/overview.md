# Project Overview: Studae — Offline-First Student Calendar with USOS Integration

Studae is an offline-first, privacy-focused productivity app for students. It provides a unified place for managing deadlines, exams, classes, and personal events — powered by secure peer-to-peer synchronization and optional USOS timetable integration.

## **[KEY-CONCEPT]** The Problem and Our Solution

Traditional student calendars often lack privacy, rely on centralized servers, and have limited offline capabilities. Furthermore, integration with university systems like USOS can be complex due to outdated APIs. Studae addresses these challenges by offering:

1. **Privacy and Security (End-to-End Encryption - E2EE):** All synced data is encrypted before leaving your device. Every device and calendar generates its own UUIDv4 and RSA key pair, and signatures are verified during sync to ensure data authenticity and track changes transparently.
2. **Decentralized Synchronization (Peer-to-Peer Sync):** Devices sync directly using WebRTC, eliminating reliance on cloud servers for user data. This also enables robust offline functionality.
3. **Flexible Calendar Management (Calendar Sharing & Offline-First):** Users can easily share individual calendars with granular access control. The app maintains full functionality without internet access, with network connectivity merely enhancing sync and integration capabilities.
4. **Seamless USOS Integration:** For universities using USOS, Studae offers optional integration to automatically import timetables. A dedicated Rust-based proxy server (`usos_proxy_server`) handles the translation of modern OAuth 2.0 PKCE authentication from the app to the legacy OAuth 1.0a required by the USOS API, securely storing secrets and signing requests on behalf of the app.

## Core Components

The project is a monorepo containing three main components, working in concert to deliver Studae's features:

1. **Flutter Mobile App (`app/`)**: The user-facing client, responsible for UI/UX, local data storage, cryptographic operations for E2EE, WebRTC-based P2P synchronization, and interaction with the `usos_proxy_server`.
2. **`usos_proxy_server`**: A secure Rust-based Backend-for-Frontend (BFF) that primarily handles the USOS integration. It translates OAuth 2.0 PKCE flows from the app into OAuth 1.0a for the USOS API, securely managing credentials and signing requests.
3. **`usos_lib`**: A shared, client-side Rust library that provides data models for the USOS API, enabling type-safe handling of API data across the Rust backend and potentially the Flutter frontend via `flutter_rust_bridge`.

This documentation provides a deeper dive into the [architecture](./architecture.md) and each of these [components](./components/).
