# `usos_proxy_server`

The `usos_proxy_server` is a standalone Rust web server that acts as a critical backend-for-frontend (BFF) for the Studae mobile app. It has two primary responsibilities: authentication and secure request proxying.

## **[CRITICAL]** Purpose 1: Authentication Translation

The first core purpose of this server is to act as an authentication broker. The USOS API requires the legacy **OAuth 1.0a** protocol, but modern mobile applications are built to use **OAuth 2.0** (with PKCE for enhanced security).

The `usos_proxy_server` bridges this gap by exposing an OAuth 2.0 interface to the Flutter app while managing the entire multi-step OAuth 1.0a dance with the USOS API in the background.

## **[CRITICAL]** Purpose 2: Secure Request-Signing Proxy

The second, equally important purpose is to act as a secure reverse proxy. The OAuth 1.0a protocol requires that every request to the USOS API be signed with a "consumer secret". This secret must **never** be stored in the mobile app.

This proxy acts as the backend for the `ProxyClient` in `usos_lib`. It is responsible for forwarding all API requests from the app to the USOS API. For each request, it uses the stored OAuth credentials (the consumer secret and the user-specific access token) to add the required cryptographic signature before sending it to USOS. The proxy implements this signing logic internally.

## Technology Stack

- **Language**: Rust
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Async Runtime**: [Tokio](https://tokio.rs/)

## API Endpoints

The server, located in the `usos_proxy_server/` directory, exposes a set of endpoints to the mobile app.

### Authentication Endpoints

These endpoints are used for the initial login flow:

- `GET /authorize`: Initiates the login process.
- `GET /callback`: The callback URL that USOS redirects to after user login.
- `POST /token`: The final step for the app to get a session token from the proxy.

### Data Proxy Endpoints (Conceptual)

To facilitate secure data fetching, the proxy must also expose its own set of endpoints that mirror the USOS API. For example:

- `POST /query/timetable/user`: The `ProxyClient` in the app would call this endpoint to get timetable data. The proxy would then make a signed request to the corresponding USOS API endpoint and return the result.

**Note:** As of the initial analysis, only the authentication endpoints are implemented. The data proxy endpoints are a necessary part of the architecture but have not yet been built.

For a detailed, step-by-step diagram of the flow, see the [System Architecture](./../architecture.md) document.

## **[CRITICAL]** Configuration and Security

As of the initial analysis, all configuration for the proxy server is **hardcoded** in `usos_proxy_server/src/main.rs`. This includes:

- OAuth Consumer Key and Secret
- USOS API URLs

**This is a major security risk.** For any production or real-world deployment, these secrets **must** be moved out of the code and loaded from a secure source, such as environment variables or a configuration file that is not committed to version control.
