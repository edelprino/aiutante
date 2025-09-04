use crate::tools;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::openai::{self, responses_api::ResponsesCompletionModel},
    tool::Tool,
};
use teloxide::{prelude::*, utils::command::BotCommands};

type OpenaiAgent = rig::agent::Agent<ResponsesCompletionModel>;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum Provider {
    OpenAI,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Metadata {
    tools: Vec<String>,
    model: Option<String>,
    provider: Option<Provider>,
}

pub struct AgentConfiguration {
    prompt: String,
    metadata: Metadata,
    name: String,
}

impl Default for AgentConfiguration {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            metadata: Metadata {
                tools: vec![],
                model: Some(openai::GPT_4O.to_string()),
                provider: Some(Provider::OpenAI),
            },
            name: "".to_string(),
        }
    }
}

impl std::fmt::Display for AgentConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let yml = serde_yaml::to_string(&self.metadata).map_err(|_| std::fmt::Error)?;
        write!(f, "---\n{yml}---\n{}", self.prompt)
    }
}

impl AgentConfiguration {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, AgentError> {
        let prompt = std::fs::read_to_string(&path)
            .map_err(|e| AgentError::new(&format!("Failed to read file: {e}")))?;
        let filename = path
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| AgentError::new("Failed to get file stem"))?;
        let mut prompt = prompt.splitn(3, "---\n");
        let yaml = prompt
            .nth(1)
            .ok_or_else(|| AgentError::new("Failed to parse prompt"))?;
        let yaml: Metadata = serde_yaml::from_str(yaml)
            .map_err(|e| AgentError::new(&format!("Failed to parse YAML: {e}")))?;
        let prompt = prompt
            .next()
            .ok_or_else(|| AgentError::new("Failed to parse prompt"))?;
        Ok(Self {
            prompt: prompt.to_string(),
            metadata: yaml,
            name: filename.to_string(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("AgentError: {message}")]
pub struct AgentError {
    message: String,
}

impl AgentError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum ObserverResponse {
    Done(String),
    Continue(String),
}

pub struct Agent {
    agent: OpenaiAgent,
}

impl Agent {
    pub fn from_configuration(
        configuration: &AgentConfiguration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = match configuration.metadata.provider {
            Some(Provider::OpenAI) | None => openai::Client::from_env(),
        };
        let builder = client.agent(
            configuration
                .metadata
                .model
                .as_deref()
                .unwrap_or(openai::GPT_4O_MINI),
        );

        let mut builder = builder.preamble(&configuration.prompt);
        for library in &configuration.metadata.tools {
            let minions_folder =
                std::env::var("MINIONS_FOLDER").expect("MINIONS_FOLDER must be set in .env");
            let path = format!("{minions_folder}/tools/{library}.yml");
            let lib_tools = tools::YamlTool::from_file(path, &configuration.name)
                .map_err(|e| AgentError::new(&format!("Failed to load tool library: {e}")))?;
            for tool in lib_tools {
                log::debug!("Registering tool from library {}: {}", library, tool.name());
                builder = builder.tool(tool);
            }
        }
        let agent = builder.build();
        Ok(Self { agent })
    }

    pub async fn run(&self, task: &str) -> Result<String, AgentError> {
        log::info!("Running agent with task: {task}");
        self.agent
            .prompt(task)
            .multi_turn(10)
            .await
            .map_err(|e| AgentError::new(&format!("Failed to run agent: {e}")))
    }

    pub async fn completions(
        &self,
        messages: Vec<rig::completion::Message>,
    ) -> Result<String, AgentError> {
        log::info!("Running agent with messages: {messages:?}");
        self.agent
            .prompt("")
            .with_history(&mut messages.clone())
            .multi_turn(10)
            .await
            .map_err(|e| AgentError::new(&format!("Failed to run agent: {e}")))
    }

    pub async fn chat(self) -> Result<(), AgentError> {
        use rig::completion::Message;
        use std::io::{self, Write};

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut chat_log = vec![];

        println!("Welcome to the chatbot! Type 'exit' to quit.");
        loop {
            print!("> ");
            stdout.flush().unwrap();

            let mut input = String::new();
            match stdin.read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();
                    if input == "exit" {
                        break;
                    }
                    let response = self
                        .agent
                        .prompt(input)
                        .with_history(&mut chat_log.clone())
                        .multi_turn(10)
                        .await
                        .map_err(|e| AgentError::new(&format!("Failed to run agent: {e}")))?;
                    chat_log.push(Message::user(input));
                    chat_log.push(Message::assistant(response.clone()));

                    println!("========================== Response ============================");
                    println!("{response}");
                    println!(
                        "================================================================\n\n"
                    );
                }
                Err(error) => println!("Error reading input: {error}"),
            }
        }

        Ok(())
    }

    pub async fn telegram(self) -> Result<(), AgentError> {
        use rig::completion::Message as RigMessage;
        use std::sync::Arc;

        let agent = Arc::new(self.agent);
        let messages = Arc::new(tokio::sync::Mutex::new(vec![]));

        let bot = Bot::from_env();

        teloxide::repl(bot, move |bot: Bot, msg: Message| {
            let agent = Arc::clone(&agent);
            let messages = Arc::clone(&messages);
            async move {
                if msg.chat.id != ChatId(24437804) {
                    return Ok(());
                }
                log::debug!("Received message: {:?}", msg.text().unwrap_or_default());

                if msg.text() == Some("/clear") {
                    let mut messages_guard = messages.lock().await;
                    messages_guard.clear();
                    drop(messages_guard);
                    bot.send_message(msg.chat.id, "Cleared chat history")
                        .await?;
                    return Ok(());
                }
                let response = agent
                    .prompt(msg.text().unwrap_or_default())
                    .with_history(&mut messages.lock().await.clone())
                    .multi_turn(10)
                    .await
                    .unwrap_or_else(|e| format!("Error: {e}"));
                log::debug!("Agent response: {response}");

                let mut messages_guard = messages.lock().await;
                messages_guard.push(RigMessage::user(msg.text().unwrap_or_default().to_string()));
                messages_guard.push(RigMessage::assistant(response.clone()));
                drop(messages_guard);

                let _ = bot.send_message(msg.chat.id, response).await;
                Ok(())
            }
        })
        .await;
        Ok(())
    }
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Clear the chat history")]
    Clear,
}
