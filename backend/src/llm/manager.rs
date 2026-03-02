use crate::config::LlmConfig;
use crate::llm::prompt::PromptManager;
use crate::llm::rig_client::{AnyAgent, AnyEmbeddingModel, RigClient};
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

    pub fn prompt(&self) -> Arc<PromptManager> {
        self.prompt.clone()
    }

    /// Build an agent without a system prompt.
    pub fn agent(&self) -> AnyAgent {
        self.rig.build_agent(None)
    }

    /// Build an agent with a system prompt (preamble).
    pub fn agent_with_preamble(&self, preamble: &str) -> AnyAgent {
        self.rig.build_agent(Some(preamble))
    }

    /// Get the embedding model for vector operations.
    pub fn embedding_model(&self) -> AnyEmbeddingModel {
        self.rig.embedding_model()
    }
}
