use crate::config::LlmConfig;
use rig::providers::openai::Client as OpenAIClient;

/// A wrapper around Rig's LLM capabilities
pub struct RigClient {
    pub openai_client: OpenAIClient,
    pub default_model: String,
}

impl RigClient {
    pub fn new(config: &LlmConfig) -> Self {
        // Handle Result from Client::new (confirmed by prior build errors)
        let openai_client =
            OpenAIClient::new(&config.api_key).expect("Failed to initialize OpenAI client");

        Self {
            openai_client,
            default_model: config.model.clone(),
        }
    }

    /// Return the underlying OpenAI client.
    /// In rig 0.31.0, traits like CompletionClient are implemented on Client.
    pub fn client(&self) -> &OpenAIClient {
        &self.openai_client
    }

    pub fn default_model(&self) -> &str {
        &self.default_model
    }
}
