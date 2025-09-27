use log::info;
use std::env;
use std::net::SocketAddr;
use warp::{http::Response, Filter};

#[tokio::main]
async fn main() {
    env_logger::init();

    // Define routes
    let reputest_get = warp::path("reputest")
        .and(warp::path::end())
        .and(warp::get())
        .map(|| {
            info!("Reputesting!");
            Response::builder()
                .header("Content-Type", "text/plain")
                .body("Reputesting!")
        });

    let reputest_post = warp::path("reputest")
        .and(warp::path::end())
        .and(warp::post())
        .map(|| {
            info!("Reputesting!");
            Response::builder()
                .header("Content-Type", "text/plain")
                .body("Reputesting!")
        });

    let health = warp::path("health")
        .and(warp::path::end())
        .and(warp::get())
        .map(|| {
            Response::builder()
                .header("Content-Type", "application/json")
                .body(r#"{"status":"healthy","service":"reputest"}"#)
        });

    let root = warp::path::end().and(warp::get()).map(|| {
        Response::builder()
            .header("Content-Type", "text/plain")
            .body("Reputest container is running!")
    });

    // Combine all routes
    let routes = reputest_get.or(reputest_post).or(health).or(root);

    // Get port from environment variable, default to 3000
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid number");

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();

    info!("Starting reputest server on {}", addr);
    warp::serve(routes).run(addr).await
}
