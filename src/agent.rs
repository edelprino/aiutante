use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::openai::{self, responses_api::ResponsesCompletionModel},
    tool::Tool,
};

use crate::tools;

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
        let prompt = std::fs::read_to_string(path)
            .map_err(|e| AgentError::new(&format!("Failed to read file: {e}")))?;
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
            let lib_tools = tools::YamlTool::from_file(path)
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
}
