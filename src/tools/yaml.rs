use rig::completion::ToolDefinition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::tools::ToolError;

pub struct YamlTool {
    name: String,
    config: ToolConfig,
    agent: String,
}

/// Configuration for a single tool argument
impl YamlTool {
    /// Create a new CustomTool from a TOML configuration file
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        agent: &str,
    ) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let config = ToolsConfig::from_file(path)?;
        Ok(config
            .0
            .into_iter()
            .map(|(name, cfg)| Self {
                name,
                config: cfg,
                agent: agent.to_string(),
            })
            .collect())
    }
}

impl rig::tool::Tool for YamlTool {
    const NAME: &'static str = "";

    type Error = ToolError;

    type Args = HashMap<String, String>;

    type Output = String;

    fn name(&self) -> String {
        self.name.clone()
    }

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let properties = if let Some(args) = &self.config.arguments {
            let props: HashMap<_, _> = args
                .iter()
                .map(|(arg_name, arg_cfg)| {
                    let mut prop = serde_json::Map::new();
                    prop.insert(
                        "type".to_string(),
                        serde_json::json!(
                            arg_cfg
                                .type_
                                .clone()
                                .unwrap_or_else(|| "string".to_string())
                        ),
                    );
                    prop.insert(
                        "description".to_string(),
                        serde_json::json!(arg_cfg.description),
                    );
                    if let Some(example) = &arg_cfg.example {
                        prop.insert("example".to_string(), serde_json::json!(example));
                    }
                    (arg_name.clone(), serde_json::Value::Object(prop))
                })
                .collect();
            props
        } else {
            HashMap::new()
        };

        ToolDefinition {
            name: self.name.clone(),
            description: self.config.description.clone(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": properties,
                "required": properties.keys().cloned().collect::<Vec<_>>(),
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut command = self.config.tool.clone();
        for (key, value) in &args {
            let placeholder = format!("{{{{{key}}}}}");
            command = command.replace(&placeholder, value);
        }

        log::debug!("[{}] Request: {:?}", self.name(), command);

        let sandbox = std::env::var("AIUTANTE_SANDBOX").unwrap_or("bash".to_string());
        let executable = sandbox.split_whitespace().next().unwrap_or("bash");
        let args = sandbox.split_whitespace().skip(1).collect::<Vec<&str>>();

        let output = std::process::Command::new(executable)
            .args(&args)
            .arg("-lc")
            .arg(command)
            .envs(std::env::vars())
            .env("AGENT", &self.agent)
            .output()
            .inspect_err(|e| log::error!("[{}] Failed to execute command: {}", self.name(), e));

        let output = match output {
            Ok(output) => output,
            Err(e) => {
                return Ok(format!("Error: {e}"));
            }
        };

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).to_string();
            log::debug!("[{}] Response: {}", self.name(), result);
            Ok(result)
        } else {
            let result = String::from_utf8_lossy(&output.stderr).to_string();
            log::error!("[{}] Error: {}", self.name(), result);
            Ok(format!("Error: {result}"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArgument {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

/// Configuration for a single tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub description: String,
    pub tool: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, ToolArgument>>,
}

/// Root configuration structure that matches the TOML format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig(HashMap<String, ToolConfig>);

impl ToolsConfig {
    /// Load tools configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: ToolsConfig = serde_yaml::from_str(&content)
            .map_err(|e| ToolError::new(&format!("Failed to parse YAML: {e}")))?;
        log::debug!("Loaded {} tools from configuration", config.0.len());
        Ok(config)
    }
}
