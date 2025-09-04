type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
use axum::{
    Router, extract,
    routing::{get, post},
};
use rig::providers::openai::Message;
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn chat_completions(
    extract::Json(payload): extract::Json<ChatCompletionRequest>,
) -> &'static str {
    dbg!(&payload);
    "Hello, World!"
}

async fn root() -> &'static str {
    "OK"
}

pub async fn run() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    let host = std::env::var("API_HOST").unwrap_or("0.0.0.0:8080".to_string());
    log::info!("Starting API server at http://{host}");
    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/", get(root))
        .route("/chat/completions", post(chat_completions))
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[derive(Clone, Deserialize, Debug)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    stream: Option<bool>,
}
