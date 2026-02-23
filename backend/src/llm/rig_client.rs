use crate::config::LlmConfig;
use rig::providers::gemini::Client as GeminiClient;

/// A wrapper around Rig's LLM capabilities using Gemini
pub struct RigClient {
    pub gemini_client: GeminiClient,
    pub default_model: String,
    pub embedding_model: String,
}

impl RigClient {
    pub fn new(config: &LlmConfig) -> Self {
        let gemini_client =
            GeminiClient::new(&config.api_key).expect("Failed to initialize Gemini client");

        Self {
            gemini_client,
            default_model: config.model.clone(),
            embedding_model: config.embedding_model.clone(),
        }
    }

    /// Return the underlying Gemini client.
    pub fn client(&self) -> &GeminiClient {
        &self.gemini_client
    }

    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    pub fn embedding_model(&self) -> &str {
        &self.embedding_model
    }
}
