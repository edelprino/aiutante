use crate::agent::{Agent, AgentConfiguration};
use clap::Parser;

mod agent;
mod api;
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
    Telegram {
        agent: String,
    },
    Api,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenvy::dotenv()
        .map_err(|e| log::warn!("Failed to read .env file: {e}"))
        .ok();

    let cli = Cli::parse();
    match cli {
        Cli::Run { agent, task } => {
            let c = AgentConfiguration::from_name(&agent).expect("Failed");
            let agent = Agent::from_configuration(&c).expect("Failed");
            let response = agent.run(&task.unwrap_or_default()).await;

            match response {
                Ok(ok) => println!("{ok}"),
                Err(err) => eprintln!("{err}"),
            }
        }
        Cli::Chat { agent } => {
            let c = AgentConfiguration::from_name(&agent).expect("Failed");
            let agent = Agent::from_configuration(&c).expect("Failed");
            agent.chat().await.expect("Failed to run chat");
        }
        Cli::Telegram { agent } => {
            let c =
                AgentConfiguration::from_name(&agent).expect("Failed to read agent configuration");
            let agent =
                Agent::from_configuration(&c).expect("Failed to create agent from configuration");
            agent.telegram().await.expect("Failed to run telegram bot");
        }
        Cli::Api => {
            api::run().await.expect("Failed to start API server");
        }
    }
}
