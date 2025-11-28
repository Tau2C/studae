use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use httpmock::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use tower::util::ServiceExt; // for `app.oneshot()`
use usos_proxy_server::{AppState, authorize, callback, token, PendingOAuth2Request, TokenRequest};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use url::Url;
use axum::routing::{get, post};
use sha2::Digest;
use base64::Engine;
use axum::body; // For axum::body::to_bytes
use serde_json;

// Helper function to create a unique random string
fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

#[tokio::test]
async fn test_oauth2_flow() {
    // 1. Setup a mock OAuth1 server
    let oauth1_mock_server = MockServer::start();

    // Configure the AppState with mocked OAuth1 URLs and a dummy consumer key/secret
    let app_state = AppState {
        oauth1_consumer_key: "test_consumer_key".to_string(),
        oauth1_consumer_secret: "test_consumer_secret".to_string(),
        oauth1_authorize_url: format!("{}/oauth/authorize", oauth1_mock_server.base_url()),
        oauth1_access_token_url: format!("{}/oauth/access_token", oauth1_mock_server.base_url()),
        proxy_base_url: "http://127.0.0.1:3000".to_string(), // This needs to match the running server
        pending_oauth2_requests: Arc::new(Mutex::new(HashMap::new())),
        issued_auth_codes: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/authorize", get(authorize))
        .route("/callback", get(callback))
        .route("/token", post(token))
        .with_state(app_state.clone()); // Clone for test client

    // Mock the OAuth1 authorize endpoint
    let oauth1_authorize_mock = oauth1_mock_server.mock(|when, then| {
        when.method(GET)
            .path("/oauth/authorize")
            .query_param("oauth_consumer_key", "test_consumer_key")
            .query_param_exists("oauth_callback");
        then.status(200)
            .header("Content-Type", "text/plain")
            .body("OAuth1 Authorization Page (mocked)");
    });

    // Mock the OAuth1 access token endpoint
    let oauth1_access_token_mock = oauth1_mock_server.mock(|when, then| {
        when.method(POST)
            .path("/oauth/access_token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body_form(
                vec![
                    ("oauth_consumer_key", "test_consumer_key"),
                    ("oauth_token", "mock_oauth1_token"), // This will be dynamic, but for exact match mock, we need fixed value.
                    ("oauth_verifier", "mock_oauth1_verifier"), // This will be dynamic too.
                ],
            );
        then.status(200)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body("oauth_token=mock_oauth1_access_token&oauth_token_secret=mock_oauth1_access_token_secret");
    });


    // 2. Simulate OAuth2 /authorize request
    let client_id = generate_random_string(16);
    let redirect_uri = "http://localhost:8080/callback".to_string();
    let state = generate_random_string(10);
    let code_verifier = generate_random_string(43); // PKCE code_verifier
    let mut hasher = sha2::Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());
    let code_challenge_method = "S256".to_string();

    let authorize_request = Request::builder()
        .method(axum::http::Method::GET) // Use axum::http::Method
        .uri(format!(
            "/authorize?client_id={}&redirect_uri={}&state={}&code_challenge={}&code_challenge_method={}",
            client_id, redirect_uri, state, code_challenge, code_challenge_method
        ))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(authorize_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FOUND);

    let location = response.headers()["location"].to_str().unwrap();
    assert!(location.starts_with(&app_state.oauth1_authorize_url));
    assert!(location.contains(&app_state.oauth1_consumer_key));
    assert!(location.contains("oauth_callback"));

    // Extract proxy_state from the redirect URL
    let redirected_url = Url::parse(location).unwrap();
    let proxy_callback_url_str = redirected_url
        .query_pairs()
        .find(|(k, _)| k == "oauth_callback")
        .unwrap()
        .1;
    let proxy_callback_url = Url::parse(&proxy_callback_url_str).unwrap();
    let proxy_state = proxy_callback_url
        .query_pairs()
        .find(|(k, _)| k == "proxy_state")
        .unwrap()
        .1
        .to_string();

    // 3. Simulate OAuth1 /callback from the OAuth1 server
    let oauth1_token = generate_random_string(20);
    let oauth1_verifier = generate_random_string(20);

    let callback_request = Request::builder()
        .method(axum::http::Method::GET) // Use axum::http::Method
        .uri(format!(
            "/callback?oauth_token={}&oauth_verifier={}&proxy_state={}",
            oauth1_token, oauth1_verifier, proxy_state
        ))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(callback_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FOUND);

    let location = response.headers()["location"].to_str().unwrap();
    assert!(location.starts_with(&redirect_uri));
    assert!(location.contains(&format!("state={}", state)));
    assert!(location.contains("code="));

    // Extract authorization code from the redirect URL
    let redirected_url = Url::parse(location).unwrap();
    let authorization_code = redirected_url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .to_string();

    // 4. Simulate OAuth2 /token request
    let token_request_body = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
        authorization_code, redirect_uri, client_id, code_verifier
    );

    let token_request = Request::builder()
        .method(axum::http::Method::POST) // Use axum::http::Method
        .uri("/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(token_request_body))
        .unwrap();

    let response = app.oneshot(token_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = body::to_bytes(response.into_body()).await.unwrap();
    let token_response: usos_proxy_server::TokenResponse = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(token_response.access_token, "mock_oauth1_access_token");
    assert_eq!(token_response.token_type, "Bearer");
    assert_eq!(token_response.expires_in, 3600);
    assert!(token_response.refresh_token.is_none());

    oauth1_authorize_mock.assert();
    oauth1_access_token_mock.assert();
}