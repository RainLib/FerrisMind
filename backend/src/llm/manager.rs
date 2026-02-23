use crate::config::LlmConfig;
use crate::llm::prompt::PromptManager;
use crate::llm::rig_client::RigClient;
use rig::client::{CompletionClient, EmbeddingsClient};
use rig::completion::CompletionModel;
use rig::embeddings::EmbeddingModel;
use std::sync::Arc;

pub struct LlmManager {
    rig: Arc<RigClient>,
    prompt: Arc<PromptManager>,
}

impl LlmManager {
    pub fn new(config: &LlmConfig) -> Self {
        let rig = Arc::new(RigClient::new(config));
        let prompt = Arc::new(PromptManager::new("prompts"));

        Self { rig, prompt }
    }

    pub fn rig(&self) -> Arc<RigClient> {
        self.rig.clone()
    }

    pub fn prompt(&self) -> Arc<PromptManager> {
        self.prompt.clone()
    }

    /// Helper to get a completion agent with the default model
    pub fn agent(
        &self,
    ) -> rig::agent::AgentBuilder<rig::providers::gemini::completion::CompletionModel> {
        let model = self.rig.default_model();
        self.rig.client().agent(model)
    }

    /// Helper to get the default embedding model
    pub fn embedding_model(&self) -> impl EmbeddingModel {
        let model = self.rig.embedding_model();
        self.rig.client().embedding_model(model)
    }
}
