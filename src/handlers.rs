//! HTTP route handlers for the reputest service.
//!
//! This module contains all the HTTP route handler functions that process
//! incoming requests and return appropriate responses.

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
};
use log::{error, info};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::db::get_all_good_vibes_degrees;

/// Handles GET requests to the `/reputest` endpoint.
///
/// This endpoint returns a simple "Reputesting!" message and logs the request.
/// It's primarily used for testing and demonstration purposes.
///
/// # Returns
///
/// A static string "Reputesting!".
pub async fn handle_reputest_get() -> &'static str {
    info!("Reputesting!");
    "Reputesting!"
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
pub async fn handle_root(State(pool): State<PgPool>) -> Result<Html<String>, (StatusCode, String)> {
    match get_all_good_vibes_degrees(&pool).await {
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

/// Escapes HTML special characters to prevent XSS attacks.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
