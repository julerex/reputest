use std::env;
use std::net::{Ipv4Addr, SocketAddr};
use warp::{http::Response, Filter};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Handle the main endpoint
    let reputest = warp::path("reputest")
        .and(warp::path::end())
        .and(warp::get().or(warp::post()))
        .map(|| {
            info!("Reputesting!");
            Response::builder()
                .header("Content-Type", "text/plain")
                .body("Reputesting!")
        });

    // Handle root path for health checks
    let health = warp::path::end()
        .and(warp::get())
        .map(|| {
            Response::builder()
                .header("Content-Type", "text/plain")
                .body("Reputest container is running!")
        });

    // Handle health check endpoint
    let health_check = warp::path("health")
        .and(warp::path::end())
        .and(warp::get())
        .map(|| {
            Response::builder()
                .header("Content-Type", "application/json")
                .body(r#"{"status":"healthy","service":"reputest"}"#)
        });

    // Combine all routes
    let routes = reputest.or(health).or(health_check);

    // Get port from environment variable, default to 8080
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a valid number");

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    
    info!("Starting reputest server on {}", addr);
    warp::serve(routes).run(addr).await
}
