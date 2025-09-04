type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
use axum::{
    Router,
    routing::{get, post},
};

async fn responses() -> &'static str {
    log::info!("Received request at /v1/resposens");
    "Hello, World!"
}

async fn root() -> &'static str {
    "OK"
}

pub async fn run() -> Result<()> {
    let host = std::env::var("API_HOST").unwrap_or("0.0.0.0:8080".to_string());
    log::info!("Starting API server at http://{host}");
    let app = Router::new()
        .route("/", get(root))
        .route("/v1/resposens", post(responses));
    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
