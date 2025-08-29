use crate::agent::{Agent, AgentConfiguration};
use clap::Parser;

mod agent;
mod tools;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Cli {
    /// Run a specific agent with an optional task
    Run {
        agent: String,
        task: Option<String>,
    },
    /// Start an interactive chat with the agent
    Chat {
        agent: String,
    },
    Create {
        agent: String,
    },
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenvy::dotenv()
        .map_err(|e| log::warn!("Failed to read .env file: {e}"))
        .ok();
    let minions_folder =
        std::env::var("MINIONS_FOLDER").expect("MINIONS_FOLDER must be set in .env");

    let cli = Cli::parse();
    match cli {
        Cli::Run { agent, task } => {
            let path = format!("{minions_folder}/{agent}.md");
            let c =
                AgentConfiguration::from_file(&path).expect("Failed to read agent configuration");
            let agent =
                Agent::from_configuration(&c).expect("Failed to create agent from configuration");
            let response = agent.run(&task.unwrap_or_default()).await;

            match response {
                Ok(ok) => println!("{ok}"),
                Err(err) => eprintln!("{err}"),
            }
        }
        Cli::Chat { agent } => {
            let path = format!("{minions_folder}/{agent}.md");
            let c =
                AgentConfiguration::from_file(&path).expect("Failed to read agent configuration");
            let agent =
                Agent::from_configuration(&c).expect("Failed to create agent from configuration");
            agent.chat().await.expect("Failed to run chat");
        }
        Cli::Create { agent } => {
            let path = format!("{minions_folder}/{agent}.md");
            if std::path::Path::new(&path).exists() {
                eprintln!("Agent {agent} already exists at {path}");
                return;
            }
            let config = AgentConfiguration::default();
            std::fs::write(&path, config.to_string()).expect("Failed to write agent template");
            println!("Created new agent {agent} at {path}");
        }
    }
}
