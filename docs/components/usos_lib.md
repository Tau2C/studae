# `usos_lib` Crate

The `usos_lib` crate is a shared Rust library that provides a flexible and type-safe client for the USOS API.

## **[KEY-CONCEPT]** The `Usos` / `UsosClient` Abstraction

The core of the library is a powerful abstraction that separates the high-level API from the low-level request mechanism.

1. **`Usos` struct**: This is the public, high-level API of the library. The mobile app's Rust code interacts with this struct to make calls like `usos.get_timetable()`. It provides an ergonomic, type-safe interface for all USOS API methods.
2. **`UsosClient` Trait**: The `Usos` struct is initialized with a concrete implementation of a `UsosClient` trait. This trait defines *how* a request is actually sent. The `Usos` struct delegates the actual sending of the request to whichever `UsosClient` it was configured with.

This design allows for two different modes of operation:

- **Proxy Mode (Default for App)**: The `Usos` struct is configured with a `ProxyClient`. When a method is called (e.g., `usos.get_timetable()`), the `ProxyClient` sends a request to the `usos_proxy_server` (e.g., `POST /query/timetable/user`). The proxy server then signs and forwards the request to the real USOS API. This is the standard, secure way for the mobile app to function.

- **Direct Mode (Testing/Advanced)**: The `Usos` struct can be configured with a `DirectClient`. This client would be initialized with all the necessary credentials (including the user's tokens and potentially the consumer secret) and would sign and send the request directly to the USOS API, bypassing the proxy. This is useful for testing or for advanced users who wish to manage their own credentials.

## Structure

The crate is located in the `usos_lib/` directory.

- **`src/lib.rs`**: This file defines the `Usos` struct, the `UsosClient` trait, and the various concrete client implementations (`ProxyClient`, `DirectClient`).
- **`src/models.rs`**: This file defines the Rust data structures (structs) that mirror the JSON objects returned by the USOS API, providing type safety.

## Usage

The primary consumer of this library is the Rust module inside the Flutter app (`app/rust/`). For the app, it is configured by default in **Proxy Mode** to ensure that no sensitive credentials are stored on the client.
