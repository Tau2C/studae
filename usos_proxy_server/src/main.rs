use axum::{
    Json, Router,
    extract::{Form, Query, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose};
use form_urlencoded;
use rand::{Rng, distributions::Alphanumeric};
use reqwest::Client;
use reqwest_oauth1::{OAuthClientProvider, Secrets};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::trace::TraceLayer;
use tracing::{Level, debug, info};
use tracing_subscriber::FmtSubscriber;
use url::Url;

// Struct to store details of a pending OAuth2 authorization request
#[derive(Debug, Clone)]
struct PendingOAuth2Request {
    client_id: String,
    redirect_uri: String,
    state: Option<String>, // OAuth2 client's original state
    code_challenge: String,
    code_challenge_method: String,
}

// State to store pending OAuth2 requests and issued authorization codes
#[derive(Debug, Clone)]
struct AppState {
    oauth1_consumer_key: String,
    oauth1_consumer_secret: String,
    oauth1_request_token_url: String, // New field for request token URL
    oauth1_authorize_url: String,
    oauth1_access_token_url: String,
    proxy_base_url: String,
    // Map: proxy_state -> PendingOAuth2Request
    pending_oauth2_requests: Arc<Mutex<HashMap<String, PendingOAuth2Request>>>,
    // Map: proxy_state -> (OAuth1 request_token, OAuth1 request_token_secret)
    oauth1_request_tokens: Arc<Mutex<HashMap<String, (String, String)>>>, // New field to store request tokens
    // Map: authorization_code -> (OAuth1 access_token, OAuth1 access_token_secret, code_challenge, code_challenge_method)
    issued_auth_codes: Arc<Mutex<HashMap<String, (String, String, String, String)>>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            oauth1_consumer_key: "hGempGK7pxkg3H5eLpzV".to_string(), // Replace with actual value
            oauth1_consumer_secret: "GUBb3X5KWK9tHzpVWDFkmfWzvpas3gwfYSdpcKfM".to_string(), // Replace with actual value
            oauth1_request_token_url: "https://usosapps.prz.edu.pl/services/oauth/request_token"
                .to_string(), // Replace with actual value
            oauth1_authorize_url: "https://usosapps.prz.edu.pl/services/oauth/authorize"
                .to_string(), // Replace with actual value
            oauth1_access_token_url: "https://usosapps.prz.edu.pl/services/oauth/access_token"
                .to_string(), // Replace with actual value
            proxy_base_url: "http://127.0.0.1:3000".to_string(), // Replace with actual value
            pending_oauth2_requests: Arc::new(Mutex::new(HashMap::new())),
            oauth1_request_tokens: Arc::new(Mutex::new(HashMap::new())), // Initialize new field
            issued_auth_codes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tokio::main]
async fn main() {
    // initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let app_state = AppState::default();

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/authorize", get(authorize))
        .route("/callback", get(callback))
        .route("/token", post(token)) // Add the token endpoint
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn authorize(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Extract OAuth2 client parameters
    let client_id = params.get("client_id").cloned().unwrap_or_default();
    let redirect_uri = params.get("redirect_uri").cloned().unwrap_or_default();
    let state = params.get("state").cloned(); // Optional OAuth2 state
    let code_challenge = params.get("code_challenge").cloned().unwrap_or_default();
    let code_challenge_method = params
        .get("code_challenge_method")
        .cloned()
        .unwrap_or_default();
    let scopes = params.get("scope").cloned(); // Optional OAuth2 scope

    info!(
        "/authorize request: client_id={}, redirect_uri={}, state={:?}, code_challenge={}, code_challenge_method={}, scopes={:?}",
        client_id, redirect_uri, state, code_challenge, code_challenge_method, scopes
    );

    // Validate essential parameters
    if client_id.is_empty() || redirect_uri.is_empty() || code_challenge.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Required parameter not present: client_id, redirect_uri, or code_challenge",
        )
            .into_response();
    }

    if client_id != "studae" {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Required parameter not present",
        )
            .into_response();
    }

    // Generate a unique proxy_state to link OAuth1 callback to this OAuth2 request
    let proxy_state: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // Store the pending OAuth2 request details
    let pending_req = PendingOAuth2Request {
        client_id,
        redirect_uri,
        state,
        code_challenge,
        code_challenge_method,
    };
    app_state
        .pending_oauth2_requests
        .lock()
        .unwrap()
        .insert(proxy_state.clone(), pending_req);

    // Construct the callback URL for this proxy (for OAuth1 request token)
    let mut proxy_callback_url_for_oauth1 =
        Url::parse(&app_state.proxy_base_url).expect("Invalid PROXY_BASE_URL constant");
    proxy_callback_url_for_oauth1.set_path("/callback");
    proxy_callback_url_for_oauth1
        .query_pairs_mut()
        .append_pair("proxy_state", &proxy_state); // Pass proxy_state to our own callback

    // Create OAuth1 secrets for signing
    let secrets = Secrets::new(
        &app_state.oauth1_consumer_key,
        &app_state.oauth1_consumer_secret,
    );

    // Build OAuth1 request token URL with callback
    let mut request_token_url = Url::parse(&app_state.oauth1_request_token_url)
        .expect("Invalid OAUTH1_REQUEST_TOKEN_URL constant");
    request_token_url
        .query_pairs_mut()
        .append_pair("oauth_callback", proxy_callback_url_for_oauth1.as_str());

    if let Some(s) = scopes {
        info!("{}", s);
        const SCOPES: [&str; 28] = [
            "adm_documents",
            "cards",
            "change_all_preferences",
            "crstests",
            "dorm_admin",
            "edit_user_attrs",
            "email",
            "events",
            "grades",
            "grades_write",
            "mailclient",
            "mobile_numbers",
            "offline_access",
            "other_emails",
            "payments",
            "personal",
            "photo",
            "placement_tests",
            "session_debugging_perms",
            "slips",
            "slips_admin",
            "staff_perspective",
            "student_exams",
            "student_exams_write",
            "studies",
            "surveys_filling",
            "surveys_reports",
            "theses_protocols_write",
        ];

        let mut scopes = Vec::new();
        for s in s.split('|') {
            if SCOPES.contains(&s) {
                info!("{}", s);
                scopes.push(s);
            }
        }

        let mut scope = String::new();
        if scopes.len() > 0 {
            scope += scopes.get(0).unwrap();
            for i in 1..scopes.len() {
                let s = scopes[i];
                scope = scope + "|" + s;
            }
        }

        request_token_url
            .query_pairs_mut()
            .append_pair("scopes", &scope);
    }

    // Make the signed request for the OAuth1 request token
    let client = Client::new();

    info!("{}", request_token_url.as_str());

    let res = client
        .oauth1(secrets)
        .get(request_token_url.as_str())
        .send()
        .await;

    let oauth1_request_token_response = match res {
        Ok(res) => {
            let text = res.text().await.unwrap_or_default();
            let parsed_params: HashMap<String, String> = form_urlencoded::parse(text.as_bytes())
                .into_owned()
                .collect();

            match (
                parsed_params.get("oauth_token"),
                parsed_params.get("oauth_token_secret"),
            ) {
                (Some(token), Some(secret)) => (token.clone(), secret.clone()),
                _ => {
                    info!("Failed to parse OAuth1 request token response: {}", text);
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to parse OAuth1 request token response: {}", text),
                    )
                        .into_response();
                }
            }
        }
        Err(e) => {
            info!("Failed to get OAuth1 request token: {}", e);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get OAuth1 request token: {}", e),
            )
                .into_response();
        }
    };

    // Store the OAuth1 request token and secret
    app_state
        .oauth1_request_tokens
        .lock()
        .unwrap()
        .insert(proxy_state.clone(), oauth1_request_token_response.clone());

    // Redirect to the OAuth1 authorization URL
    let mut auth1_redirect_url =
        Url::parse(&app_state.oauth1_authorize_url).expect("Invalid OAUTH1_AUTHORIZE_URL constant");
    auth1_redirect_url
        .query_pairs_mut()
        .append_pair("oauth_token", &oauth1_request_token_response.0); // Use the obtained request token

    info!(
        "Redirecting to OAuth1 Authorize: {}",
        auth1_redirect_url.as_str()
    );
    Redirect::to(auth1_redirect_url.as_str()).into_response()
}

// Struct to deserialize OAuth1 access token response
#[derive(Debug, Deserialize)]
struct OAuth1AccessTokenResponse {
    oauth_token: String,
    oauth_token_secret: String,
}

async fn callback(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let oauth_token = params.get("oauth_token").cloned().unwrap_or_default();
    let oauth_verifier = params.get("oauth_verifier").cloned().unwrap_or_default();
    let proxy_state = params.get("proxy_state").cloned().unwrap_or_default();

    info!(
        "/callback request: oauth_token={}, oauth_verifier={}, proxy_state={}",
        oauth_token, oauth_verifier, proxy_state
    );

    if oauth_token.is_empty() || oauth_verifier.is_empty() || proxy_state.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Missing required OAuth1 callback parameters: oauth_token, oauth_verifier, or proxy_state",
        )
            .into_response();
    }

    // Retrieve the pending OAuth2 request using proxy_state
    let pending_req = app_state
        .pending_oauth2_requests
        .lock()
        .unwrap()
        .remove(&proxy_state);

    let pending_req = match pending_req {
        Some(req) => req,
        None => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid or expired proxy_state",
            )
                .into_response();
        }
    };

    // Retrieve the OAuth1 request token and secret
    let (oauth1_req_token, oauth1_req_token_secret) = match app_state
        .oauth1_request_tokens
        .lock()
        .unwrap()
        .remove(&proxy_state)
    {
        Some(tokens) => tokens,
        None => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid or expired OAuth1 request token (from proxy_state)",
            )
                .into_response();
        }
    };

    // Exchange OAuth1 request token for access token
    let secrets = Secrets::new(
        &app_state.oauth1_consumer_key,
        &app_state.oauth1_consumer_secret,
    )
    .token(&oauth1_req_token, &oauth1_req_token_secret);

    let client = Client::new();
    let res = client
        .oauth1(secrets)
        .post(&app_state.oauth1_access_token_url)
        .form(&[("oauth_verifier", oauth_verifier.as_str())])
        .send()
        .await;

    let oauth1_access_token_response = match res {
        Ok(res) => {
            let text = res.text().await.unwrap_or_default();
            // OAuth1 access token response is usually url-encoded, not JSON
            let parsed_params: HashMap<String, String> = form_urlencoded::parse(text.as_bytes())
                .into_owned()
                .collect();

            match (
                parsed_params.get("oauth_token"),
                parsed_params.get("oauth_token_secret"),
            ) {
                (Some(token), Some(secret)) => OAuth1AccessTokenResponse {
                    oauth_token: token.clone(),
                    oauth_token_secret: secret.clone(),
                },
                _ => {
                    info!("Failed to parse OAuth1 access token response: {}", text);
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to parse OAuth1 access token response: {}", text),
                    )
                        .into_response();
                }
            }
        }
        Err(e) => {
            info!(
                "Failed to exchange OAuth1 request token for access token: {}",
                e
            );
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Failed to exchange OAuth1 request token for access token: {}",
                    e
                ),
            )
                .into_response();
        }
    };

    // Generate a unique authorization code for the OAuth2 flow
    let authorization_code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // Store the OAuth1 access token and secret, associated with the authorization code,
    // along with the code_challenge and code_challenge_method for PKCE verification
    app_state.issued_auth_codes.lock().unwrap().insert(
        authorization_code.clone(),
        (
            oauth1_access_token_response.oauth_token,
            oauth1_access_token_response.oauth_token_secret,
            pending_req.code_challenge,
            pending_req.code_challenge_method,
        ),
    );

    // Construct the redirect URL back to the OAuth2 client
    let mut redirect_to_client = Url::parse(&pending_req.redirect_uri)
        .expect("Invalid redirect_uri from pending OAuth2 request");
    redirect_to_client
        .query_pairs_mut()
        .append_pair("code", &authorization_code);
    if let Some(state) = pending_req.state {
        redirect_to_client
            .query_pairs_mut()
            .append_pair("state", &state);
    }

    // Redirect to the OAuth2 client
    Redirect::to(redirect_to_client.as_str()).into_response()
}

// Struct to deserialize the OAuth2 token request
#[derive(Debug, Deserialize)]
struct TokenRequest {
    grant_type: String,
    code: String,
    redirect_uri: String,
    client_id: String,
    code_verifier: String,
}

// Struct to serialize the OAuth2 token response
#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: Option<String>,
}

async fn token(
    State(app_state): State<AppState>,
    Form(params): Form<TokenRequest>,
) -> impl IntoResponse {
    info!("/token request: {:?}", params);

    // Validate grant_type
    if params.grant_type != "authorization_code" {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Unsupported grant_type",
        )
            .into_response();
    }

    // Retrieve OAuth1 access token and PKCE details from issued_auth_codes
    let (
        oauth1_access_token,
        _oauth1_access_token_secret,
        stored_code_challenge,
        stored_code_challenge_method,
    ) = match app_state
        .issued_auth_codes
        .lock()
        .unwrap()
        .remove(&params.code)
    {
        Some(tokens) => tokens,
        None => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid or expired authorization code",
            )
                .into_response();
        }
    };

    // PKCE code_challenge verification
    let derived_code_challenge = match stored_code_challenge_method.as_str() {
        "S256" => {
            let mut hasher = Sha256::new();
            hasher.update(params.code_verifier.as_bytes());
            general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize())
        }
        "plain" => params.code_verifier,
        _ => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                "Unsupported code_challenge_method",
            )
                .into_response();
        }
    };

    if derived_code_challenge != stored_code_challenge {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "PKCE code_verifier mismatch",
        )
            .into_response();
    }

    // Return a new OAuth2 access token
    let response = TokenResponse {
        access_token: oauth1_access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1 hour
        refresh_token: None,
    };

    Json(response).into_response()
}
