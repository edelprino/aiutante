mod yaml;
pub use yaml::YamlTool;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ToolError {
    message: String,
}

impl ToolError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}
