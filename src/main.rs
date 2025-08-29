use crate::agent::{Agent, AgentConfiguration};
use clap::Parser;

mod agent;
mod tools;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Cli {
    /// Run a specific agent with an optional task
    Run { agent: String, task: Option<String> },
    /// Start an interactive chat with the agent
    Chat { agent: String },
    /// List of all available agents
    List {},
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
        Cli::List {} => {
            let paths = std::fs::read_dir(&minions_folder)
                .expect("Failed to read minions folder")
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.extension()? == "md" {
                        Some(path.file_stem()?.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if paths.is_empty() {
                println!("No agents found in {minions_folder}");
            } else {
                println!("Available agents:");
                for path in paths {
                    println!("- {path}");
                }
            }
        }
    }
}
