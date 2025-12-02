# Mobile App (`app/`)

The mobile app is the user-facing component of the Studae project. It is built with Flutter and provides the modern user experience for interacting with the USOS system.

## **[KEY-CONCEPT]** Technology Stack

- **Framework**: [Flutter](https://flutter.dev/)
- **Language**: [Dart](https://dart.dev/)
- **Native Integration**: [flutter_rust_bridge](https://cjycode.com/flutter_rust_bridge/) is used to bridge native Rust logic to the Dart frontend.

## **[CRITICAL]** Communication Flow

The app's embedded Rust module (`app/rust/`) handles all communication with the backend. It uses the **`usos_lib`** crate.

To protect sensitive credentials, the mobile app **must not** communicate directly with the USOS API. Instead, it communicates exclusively with the `usos_proxy_server` for all its needs.

## Structure and Features

The app's codebase is located in the `app/` directory.

- **Entry Point**: The main entry point for the app is `lib/main.dart`.
- **UI**: The user interface is built with standard Flutter widgets.
- **State Management**: The project currently uses a simple `StatefulWidget` and `setState` approach.
- **Rust Integration**: The native Rust code is in `app/rust/`. It is called from Dart via `flutter_rust_bridge`. Its primary purpose is to use the `usos_lib` crate to perform API calls.
