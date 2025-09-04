type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
use axum::{
    Router, extract,
    response::IntoResponse,
    routing::{get, post},
};
use axum_streams::*;
use futures::prelude::*;
use rig::providers::openai::Message;
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::agent::{Agent, AgentConfiguration};

#[derive(serde::Serialize)]
struct ChatCompletionChunk {
    object: String,
    created: u64,
    choices: Vec<Choice>,
}

#[derive(serde::Serialize)]
struct Choice {
    index: i32,
    delta: Delta,
    finish_reason: Option<String>,
}

#[derive(serde::Serialize)]
struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

impl ChatCompletionChunk {
    fn new(content: Option<String>, role: Option<String>, finish_reason: Option<String>) -> Self {
        Self {
            object: "chat.completion.chunk".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            choices: vec![Choice {
                index: 0,
                delta: Delta { role, content },
                finish_reason,
            }],
        }
    }

    fn stop() -> Self {
        Self::new(None, None, Some("stop".to_string()))
    }

    fn content(content: &str) -> Self {
        Self::new(Some(content.to_string()), None, None)
    }

    fn start(role: &str) -> Self {
        Self::new(None, Some(role.to_string()), None)
    }
}

async fn chat_completions(
    extract::Json(payload): extract::Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    let agent = payload.model;
    let messages: Vec<rig::message::Message> = payload
        .messages
        .iter()
        .skip(1)
        .map(|m| m.clone().try_into().unwrap())
        .collect();

    let minions_folder =
        std::env::var("MINIONS_FOLDER").expect("MINIONS_FOLDER must be set in .env");
    let path = format!("{minions_folder}/{agent}.md");
    let c = AgentConfiguration::from_file(&path).expect("Failed to read agent configuration");
    let agent = Agent::from_configuration(&c).expect("Failed to create agent from configuration");
    let response = agent
        .completions(messages)
        .await
        .expect("Failed to get completions");

    let chunks = vec![
        ChatCompletionChunk::start("assistant"),
        ChatCompletionChunk::content(&response),
        ChatCompletionChunk::stop(),
    ];
    let stream = stream::iter(chunks)
        .map(|chunk| {
            let json = serde_json::to_string(&chunk).unwrap();
            format!("data: {}\n\n", json)
        })
        .chain(stream::once(async { "data: [DONE]\n\n".to_string() }));
    axum::response::Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .body(StreamBodyAs::text(stream))
        .unwrap()
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
