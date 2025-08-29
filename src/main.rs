use crate::agent::{Agent, AgentConfiguration};
use clap::Parser;

mod agent;
mod tools;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Cli {
    Run { agent: String, task: Option<String> },
    Chat { agent: String },
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
            let path = format!("/Users/edelprino/Knowledge/Risorse/Agents/{agent}.md");
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
            let path = format!("/Users/edelprino/Knowledge/Risorse/Agents/{agent}.md");
            let c =
                AgentConfiguration::from_file(&path).expect("Failed to read agent configuration");
            let agent =
                Agent::from_configuration(&c).expect("Failed to create agent from configuration");
            agent.chat().await.expect("Failed to run chat");
        }
    }
}
