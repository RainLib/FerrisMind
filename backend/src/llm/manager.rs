use crate::config::LlmConfig;
use crate::llm::rig_client::RigClient;
use std::sync::Arc;

pub struct LlmManager {
    rig: Arc<RigClient>,
}

impl LlmManager {
    pub fn new(config: &LlmConfig) -> Self {
        let rig = Arc::new(RigClient::new(config));

        Self { rig }
    }

    pub fn rig(&self) -> Arc<RigClient> {
        self.rig.clone()
    }
}
