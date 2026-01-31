//! HTTP route handlers for the reputest service.
//!
//! This module contains all the HTTP route handler functions that process
//! incoming requests and return appropriate responses.

use axum::{
    body::Bytes,
    extract::{Query, Request, State},
    http::{header, HeaderMap, StatusCode},
    response::{AppendHeaders, Html, IntoResponse, Json, Redirect},
};
use log::{error, info, warn};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::config::get_allowed_username;
use crate::db::{
    create_session, delete_session, get_all_good_vibes_degrees, get_session_by_id, WebSession,
};
use crate::oauth::{
    build_authorization_url, exchange_authorization_code, generate_code_challenge,
    generate_code_verifier, generate_oauth_state,
};

/// Application state for routes that need pool and OAuth config.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub base_url: Option<String>,
    pub oauth_client_id: Option<String>,
    pub oauth_client_secret: Option<String>,
}

/// Parses a cookie value from the Cookie header by name.
fn get_cookie_from_header(cookie_header: Option<&axum::http::HeaderValue>, name: &str) -> Option<String> {
    let s = cookie_header?.to_str().ok()?;
    for part in s.split("; ") {
        let mut it = part.splitn(2, '=');
        if it.next()? == name {
            return it.next().map(|v| v.trim_matches('"').to_string());
        }
    }
    None
}

/// Handles GET /reputest: OAuth callback when ?code=&state= are present, otherwise "Reputesting!".
/// The callback URI is https://reputest.fly.dev/reputest — X redirects here after authorization.
pub async fn handle_reputest_get(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
    request: Request,
) -> axum::response::Response {
    let is_oauth_callback = query
        .code
        .as_deref()
        .map_or(false, |s| !s.is_empty())
        && query
            .state
            .as_deref()
            .map_or(false, |s| !s.is_empty());
    if is_oauth_callback {
        oauth_callback_response(state, query, request).await.into_response()
    } else {
        info!("Reputesting!");
        "Reputesting!".into_response()
    }
}

/// Handles POST requests to the `/reputest` endpoint.
///
/// This endpoint returns a simple "Reputesting!" message and logs the request.
/// It's primarily used for testing and demonstration purposes.
///
/// # Returns
///
/// A static string "Reputesting!".
pub async fn handle_reputest_post() -> &'static str {
    info!("Reputesting!");
    "Reputesting!"
}

/// Handles GET requests to the `/health` endpoint.
///
/// This endpoint provides a health check for the service, returning the current
/// status and service name. It's commonly used by load balancers and monitoring
/// systems to verify that the service is running and responsive.
///
/// # Returns
///
/// A JSON response containing:
/// - `status`: Always "healthy" when the service is running
/// - `service`: The service name "reputest"
///
/// # Example Response
///
/// ```json
/// {
///   "status": "healthy",
///   "service": "reputest"
/// }
/// ```
pub async fn handle_health() -> Json<Value> {
    Json(json!({"status": "healthy", "service": "reputest"}))
}

/// Handles GET requests to the root `/` endpoint.
///
/// This endpoint displays a table with data from the view_all_good_vibes_degrees view.
/// It shows sensor, emitter, and all four degree path counts.
///
/// # Returns
///
/// An HTML page with a table displaying the view data.
pub async fn handle_root(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    match get_all_good_vibes_degrees(&state.pool).await {
        Ok(rows) => {
            let mut html = String::from(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reputest - Good Vibes</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            max-width: 1600px;
            margin: 0 auto;
            background-color: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            margin-top: 0;
        }
        table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }
        th, td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        th {
            background-color: #f8f9fa;
            font-weight: 600;
            color: #555;
        }
        tr:hover {
            background-color: #f8f9fa;
        }
        .count {
            text-align: right;
            font-variant-numeric: tabular-nums;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Good Vibes</h1>
        <table>
            <thead>
                <tr>
                    <th>sensor</th>
                    <th>sensor name</th>
                    <th>emitter</th>
                    <th>emitter name</th>
                    <th class="count">one-degree-vibe-count</th>
                    <th class="count">two-degree-vibe-count</th>
                    <th class="count">three-degree-vibe-count</th>
                    <th class="count">four-degree-vibe-count</th>
                </tr>
            </thead>
            <tbody>
"#,
            );

            for row in rows {
                html.push_str(&format!(
                    "                <tr>\n                    <td>{}</td>\n                    <td>{}</td>\n                    <td>{}</td>\n                    <td>{}</td>\n                    <td class=\"count\">{}</td>\n                    <td class=\"count\">{}</td>\n                    <td class=\"count\">{}</td>\n                    <td class=\"count\">{}</td>\n                </tr>\n",
                    html_escape(&row.sensor_username),
                    html_escape(&row.sensor_name),
                    html_escape(&row.emitter_username),
                    html_escape(&row.emitter_name),
                    row.degree_one_path_count,
                    row.degree_two_path_count,
                    row.degree_three_path_count,
                    row.degree_four_path_count
                ));
            }

            html.push_str(
                r#"            </tbody>
        </table>
    </div>
</body>
</html>"#,
            );

            Ok(Html(html))
        }
        Err(e) => {
            // SECURITY: Log detailed error server-side but return generic message to client
            error!("Failed to query view_all_good_vibes_degrees: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occurred. Please try again later.".to_string(),
            ))
        }
    }
}

/// GET /login — Login page with "Login with X" link. If already logged in, redirect to /playground.
pub async fn handle_login(
    State(state): State<AppState>,
    request: Request,
) -> impl IntoResponse {
    let cookie_header = request.headers().get(header::COOKIE);
    if let Some(sid) = get_cookie_from_header(cookie_header, "session_id") {
        if let Ok(id) = sqlx::types::Uuid::parse_str(&sid) {
            if get_session_by_id(&state.pool, id).await.ok().flatten().is_some() {
                return Redirect::to("/playground").into_response();
            }
        }
    }
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Login - Reputest</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 400px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        h1 { color: #333; margin-top: 0; }
        a.btn { display: inline-block; padding: 12px 24px; background: #1DA1F2; color: white; text-decoration: none; border-radius: 8px; font-weight: 600; }
        a.btn:hover { background: #1a91da; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Login</h1>
        <p><a href="/login/start" class="btn">Login with X</a></p>
    </div>
</body>
</html>"#;
    Html(html.to_string()).into_response()
}

/// GET /login/start — Start OAuth flow: set PKCE cookies and redirect to X.
pub async fn handle_login_start(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let base_url = state
        .base_url
        .as_deref()
        .ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "Web login not configured (BASE_URL).".to_string(),
        ))?;
    let client_id = state
        .oauth_client_id
        .as_deref()
        .ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "Web login not configured (XAPI_CLIENT_ID).".to_string(),
        ))?;
    let redirect_uri = base_url.trim_end_matches('/').to_string();
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let oauth_state = generate_oauth_state();

    let auth_url = build_authorization_url(client_id, &redirect_uri, &code_challenge, &oauth_state);

    // Set cookies: 10 min max-age, Path=/, HttpOnly, SameSite=Lax
    let secure_attr = if base_url.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };
    let state_cookie = format!(
        "oauth_state={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=600{}",
        oauth_state, secure_attr
    );
    let verifier_cookie = format!(
        "oauth_code_verifier={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=600{}",
        code_verifier, secure_attr
    );

    Ok((
        AppendHeaders([
            (
                header::SET_COOKIE,
                axum::http::HeaderValue::try_from(state_cookie).unwrap(),
            ),
            (
                header::SET_COOKIE,
                axum::http::HeaderValue::try_from(verifier_cookie).unwrap(),
            ),
        ]),
        Redirect::to(&auth_url),
    ))
}

/// Query params for OAuth callback.
#[derive(serde::Deserialize)]
pub struct OAuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

/// Performs the OAuth callback: exchange code for tokens, create session, redirect to /playground.
/// Used when GET /reputest is hit with ?code=&state= (callback URI is https://reputest.fly.dev/reputest).
async fn oauth_callback_response(
    state: AppState,
    query: OAuthCallbackQuery,
    request: Request,
) -> impl IntoResponse {
    if state.base_url.is_none()
        || state.oauth_client_id.is_none()
        || state.oauth_client_secret.is_none()
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Web login not configured.".to_string(),
        )
            .into_response();
    }
    let cookie_header = request.headers().get(header::COOKIE);
    let stored_state = get_cookie_from_header(cookie_header, "oauth_state");
    let code_verifier = get_cookie_from_header(cookie_header, "oauth_code_verifier");

    let code = match &query.code {
        Some(c) if !c.is_empty() => c.as_str(),
        _ => {
            warn!("OAuth callback missing code");
            return (
                StatusCode::BAD_REQUEST,
                "Missing code parameter".to_string(),
            )
                .into_response();
        }
    };
    let state_param = match &query.state {
        Some(s) => s.as_str(),
        _ => {
            warn!("OAuth callback missing state");
            return (
                StatusCode::BAD_REQUEST,
                "Missing state parameter".to_string(),
            )
                .into_response();
        }
    };
    let (stored_state, code_verifier) = match (stored_state, code_verifier) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            warn!("OAuth callback missing state or code_verifier cookie");
            return (
                StatusCode::BAD_REQUEST,
                "Missing OAuth state. Please try logging in again.".to_string(),
            )
                .into_response();
        }
    };
    if state_param != stored_state {
        warn!("OAuth state mismatch");
        return (
            StatusCode::BAD_REQUEST,
            "Invalid state. Please try logging in again.".to_string(),
        )
            .into_response();
    }

    let redirect_uri = state
        .base_url
        .as_deref()
        .unwrap_or("")
        .trim_end_matches('/')
        .to_string();
    let client_id = state.oauth_client_id.as_deref().unwrap_or("");
    let client_secret = state.oauth_client_secret.as_deref().unwrap_or("");

    let (access_token, refresh_token) = match exchange_authorization_code(
        client_id,
        client_secret,
        redirect_uri.as_str(),
        code,
        &code_verifier,
    )
    .await
    {
        Ok(tokens) => tokens,
        Err(e) => {
            error!("OAuth token exchange failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token exchange failed. Please try again.".to_string(),
            )
                .into_response();
        }
    };

    // Get current user via /2/users/me
    let client = reqwest::Client::new();
    let me_url = "https://api.twitter.com/2/users/me?user.fields=id,username";
    let me_response = client
        .get(me_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await;
    let (user_id, username) = match me_response {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            let json: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
            let data = json.get("data");
            let user_id = data.and_then(|d| d.get("id")).and_then(|v| v.as_str()).map(String::from);
            let username = data.and_then(|d| d.get("username")).and_then(|v| v.as_str()).map(String::from);
            match (user_id, username) {
                (Some(id), Some(un)) => (id, un),
                _ => {
                    error!("Could not parse /2/users/me response");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Could not get user info.".to_string(),
                    )
                        .into_response();
                }
            }
        }
        _ => {
            error!("/2/users/me request failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not get user info.".to_string(),
            )
                .into_response();
        }
    };

    if let Some(allowed) = get_allowed_username() {
        if username != allowed {
            warn!("Login rejected: username {} not in ALLOWED_USERNAME", username);
            return (
                StatusCode::FORBIDDEN,
                "You are not allowed to log in to this application.".to_string(),
            )
                .into_response();
        }
    }

    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);
    let session_id = match create_session(
        &state.pool,
        &user_id,
        &username,
        &access_token,
        refresh_token.as_deref(),
        expires_at,
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to create session: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create session.".to_string(),
            )
                .into_response();
        }
    };

    let secure_attr = if redirect_uri.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };
    let session_cookie = format!(
        "session_id={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800{}",
        session_id, secure_attr
    );
    // Clear OAuth cookies
    let clear_state = "oauth_state=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    let clear_verifier = "oauth_code_verifier=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";

    (
        AppendHeaders([
            (
                header::SET_COOKIE,
                axum::http::HeaderValue::try_from(session_cookie).unwrap(),
            ),
            (
                header::SET_COOKIE,
                axum::http::HeaderValue::try_from(clear_state).unwrap(),
            ),
            (
                header::SET_COOKIE,
                axum::http::HeaderValue::try_from(clear_verifier).unwrap(),
            ),
        ]),
        Redirect::to("/playground"),
    )
        .into_response()
}

/// Loads session from Cookie header if present and valid. Returns None if missing or expired.
async fn get_session_from_headers(
    state: &AppState,
    headers: &HeaderMap,
) -> Option<WebSession> {
    let cookie_header = headers.get(header::COOKIE);
    let sid = get_cookie_from_header(cookie_header, "session_id")?;
    let id = sqlx::types::Uuid::parse_str(&sid).ok()?;
    get_session_by_id(&state.pool, id).await.ok().flatten()
}

/// GET /playground — API explorer page (requires login).
pub async fn handle_playground_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> axum::response::Response {
    let session = get_session_from_headers(&state, &headers).await;
    match session {
        None => Redirect::to("/login").into_response(),
        Some(session) => {
            let html = playground_html(&session.username, "", "GET", "", None);
            Html(html).into_response()
        }
    }
}

/// Form body for playground POST.
#[derive(serde::Deserialize)]
pub struct PlaygroundForm {
    path: String,
    method: String,
    #[serde(default)]
    body: String,
}

/// POST /playground — Run X API request and re-render page with response.
pub async fn handle_playground_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    let session = get_session_from_headers(&state, &headers).await;
    let session = match session {
        None => return Redirect::to("/login").into_response(),
        Some(s) => s,
    };

    let form: PlaygroundForm = match serde_urlencoded::from_bytes(&body) {
        Ok(f) => f,
        Err(_) => {
            return playground_response(
                &session.username,
                &PlaygroundForm {
                    path: String::new(),
                    method: "GET".to_string(),
                    body: String::new(),
                },
                Some(Err("Invalid form data".to_string())),
            )
            .into_response()
        }
    };

    let path = form.path.trim().trim_start_matches('/');
    if path.is_empty() {
        return playground_response(&session.username, &form, Some(Err("Path is required".to_string())))
            .into_response();
    }
    if !path.starts_with("2/") {
        return playground_response(
            &session.username,
            &form,
            Some(Err("Path must start with 2/ (Twitter API v2)".to_string())),
        )
        .into_response();
    }
    if path.contains("//") {
        return playground_response(
            &session.username,
            &form,
            Some(Err("Invalid path".to_string())),
        )
        .into_response();
    }

    let url = format!("https://api.twitter.com/{}", path);
    let method = form.method.to_uppercase();
    let method = if method == "POST" {
        reqwest::Method::POST
    } else {
        reqwest::Method::GET
    };

    let req = reqwest::Client::new()
        .request(method.clone(), &url)
        .header("Authorization", format!("Bearer {}", session.access_token));

    let response = if method == reqwest::Method::POST && !form.body.trim().is_empty() {
        req.body(form.body.clone()).send().await
    } else {
        req.send().await
    };

    let result = match response {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_else(|_| String::new());
            Ok((status, text))
        }
        Err(e) => Err(e.to_string()),
    };

    Html(playground_response(&session.username, &form, Some(result))).into_response()
}

fn playground_html(
    username: &str,
    path: &str,
    method: &str,
    body: &str,
    response: Option<Result<(u16, String), String>>,
) -> String {
    let (status_line, response_body) = match response {
        None => (String::new(), String::new()),
        Some(Ok((status, resp_body))) => (format!("Status: {}", status), html_escape(&resp_body)),
        Some(Err(e)) => (String::new(), html_escape(&e)),
    };
    let path_attr = html_escape(path);
    let method_get = if method == "GET" { " selected" } else { "" };
    let method_post = if method == "POST" { " selected" } else { "" };
    let body_value = html_escape(body);
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>X API Playground - Reputest</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 900px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        h1 {{ color: #333; margin-top: 0; }}
        label {{ display: block; margin-top: 12px; font-weight: 600; color: #555; }}
        input[type="text"], select, textarea {{ width: 100%; padding: 10px; margin-top: 4px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; }}
        textarea {{ min-height: 120px; font-family: monospace; }}
        .response {{ margin-top: 20px; padding: 12px; background: #f8f9fa; border-radius: 4px; font-family: monospace; white-space: pre-wrap; word-break: break-all; }}
        .meta {{ margin-bottom: 8px; color: #666; }}
        button {{ margin-top: 12px; padding: 10px 20px; background: #1DA1F2; color: white; border: none; border-radius: 8px; font-weight: 600; cursor: pointer; }}
        button:hover {{ background: #1a91da; }}
        a {{ color: #1DA1F2; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>X API Playground</h1>
        <p>Logged in as <strong>{}</strong> · <a href="/logout">Logout</a></p>
        <form method="post" action="/playground">
            <label>X API path (e.g. 2/users/me)</label>
            <input type="text" name="path" value="{}" placeholder="2/users/me" />
            <label>Method</label>
            <select name="method">
                <option value="GET"{}>GET</option>
                <option value="POST"{}>POST</option>
            </select>
            <label>Body (JSON, for POST)</label>
            <textarea name="body" placeholder="{{}}">{}</textarea>
            <button type="submit">Call API</button>
        </form>
        <div class="response">
            <div class="meta">{}</div>
            <pre>{}</pre>
        </div>
    </div>
</body>
</html>"#,
        html_escape(username),
        path_attr,
        method_get,
        method_post,
        body_value,
        status_line,
        response_body
    )
}

fn playground_response(
    username: &str,
    form: &PlaygroundForm,
    result: Option<Result<(u16, String), String>>,
) -> String {
    playground_html(
        username,
        &form.path,
        &form.method,
        &form.body,
        result,
    )
}

/// GET /logout — Delete session and redirect to /login.
pub async fn handle_logout(
    State(state): State<AppState>,
    request: Request,
) -> impl IntoResponse {
    let cookie_header = request.headers().get(header::COOKIE);
    let sid = get_cookie_from_header(cookie_header, "session_id");
    if let Some(s) = sid {
        if let Ok(id) = sqlx::types::Uuid::parse_str(&s) {
            let _ = delete_session(&state.pool, id).await;
        }
    }
    let clear_cookie = "session_id=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    (
        AppendHeaders([(
            header::SET_COOKIE,
            axum::http::HeaderValue::try_from(clear_cookie).unwrap(),
        )]),
        Redirect::to("/login"),
    )
        .into_response()
}

/// Escapes HTML special characters to prevent XSS attacks.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
