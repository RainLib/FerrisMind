use crate::config::LlmConfig;
use crate::graph::context::StageSender;
use axum::response::sse::Event;
use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::completion::{Prompt, PromptError};
use rig::providers::{anthropic, deepseek, gemini, openai};
use tracing::{info, warn};

const LLM_MAX_RETRIES: u32 = 3;

pub enum AnyAgent {
    Gemini(Agent<gemini::completion::CompletionModel>),
    OpenAi(Agent<openai::completion::CompletionModel>),
    Anthropic(Agent<anthropic::completion::CompletionModel>),
    DeepSeek(Agent<deepseek::CompletionModel>),
}

impl AnyAgent {
    pub async fn prompt(&self, text: &str) -> Result<String, PromptError> {
        match self {
            Self::Gemini(a) => a.prompt(text).await,
            Self::OpenAi(a) => a.prompt(text).await,
            Self::Anthropic(a) => a.prompt(text).await,
            Self::DeepSeek(a) => a.prompt(text).await,
        }
    }

    pub async fn prompt_with_retry(&self, text: &str, operation: &str) -> Result<String, String> {
        let mut last_err = String::new();
        for attempt in 1..=LLM_MAX_RETRIES + 1 {
            match self.prompt(text).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_err = e.to_string();
                    if attempt <= LLM_MAX_RETRIES {
                        warn!(
                            "{} failed (attempt {}/{}): {}",
                            operation,
                            attempt,
                            LLM_MAX_RETRIES + 1,
                            last_err
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(attempt as u64)).await;
                    }
                }
            }
        }
        Err(format!(
            "{} failed after {} attempts: {}",
            operation,
            LLM_MAX_RETRIES + 1,
            last_err
        ))
    }

    pub async fn stream_to_sse(
        &self,
        prompt_text: &str,
        tx: &StageSender,
        operation: &str,
    ) -> Result<String, String> {
        let mut last_err = String::new();
        for attempt in 1..=LLM_MAX_RETRIES + 1 {
            let result = match self {
                Self::Gemini(a) => do_stream(a, prompt_text, tx).await,
                Self::OpenAi(a) => do_stream(a, prompt_text, tx).await,
                Self::Anthropic(a) => do_stream(a, prompt_text, tx).await,
                Self::DeepSeek(a) => do_stream(a, prompt_text, tx).await,
            };

            match result {
                StreamResult::Ok(text) => return Ok(text),
                StreamResult::PartialError(err) => {
                    return Err(format!("{} streaming error (partial): {}", operation, err));
                }
                StreamResult::RetryableError(err) => {
                    last_err = err;
                    if attempt <= LLM_MAX_RETRIES {
                        warn!(
                            "{} failed (attempt {}/{}): {}",
                            operation,
                            attempt,
                            LLM_MAX_RETRIES + 1,
                            last_err
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(attempt as u64)).await;
                        continue;
                    }
                }
            }
        }
        Err(format!(
            "{} failed after {} attempts: {}",
            operation,
            LLM_MAX_RETRIES + 1,
            last_err
        ))
    }
}

enum StreamResult {
    Ok(String),
    PartialError(String),
    RetryableError(String),
}

async fn do_stream<M>(
    agent: &Agent<M>,
    prompt_text: &str,
    tx: &StageSender,
) -> StreamResult
where
    M: rig::completion::CompletionModel + 'static,
    M::StreamingResponse: Send + Sync + Clone + Unpin + rig::completion::GetTokenUsage,
{
    use futures::StreamExt;
    use rig::agent::MultiTurnStreamItem;
    use rig::streaming::{StreamedAssistantContent, StreamingPrompt};

    let mut stream = agent.stream_prompt(prompt_text.to_string()).await;
    let mut full_response = String::new();
    let mut has_content = false;
    let mut stream_error = None;

    while let Some(item) = stream.next().await {
        match item {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(
                text,
            ))) => {
                has_content = true;
                full_response.push_str(&text.text);
                let _ = tx
                    .send(Ok(Event::default().event("answer").data(&text.text)))
                    .await;
            }
            Ok(MultiTurnStreamItem::FinalResponse(_)) => break,
            Err(e) => {
                stream_error = Some(e.to_string());
                break;
            }
            _ => {}
        }
    }

    match stream_error {
        Some(err) if has_content => StreamResult::PartialError(err),
        Some(err) => StreamResult::RetryableError(err),
        None => StreamResult::Ok(full_response),
    }
}

// ─── Provider clients ───

pub enum ProviderClient {
    Gemini(gemini::Client),
    OpenAi(openai::CompletionsClient),
    Anthropic(anthropic::Client),
    DeepSeek(deepseek::Client),
}

pub struct RigClient {
    completion_client: ProviderClient,
    embedding_client: EmbeddingClient,
    pub default_model: String,
    pub embedding_model: String,
}

impl RigClient {
    pub fn new(config: &LlmConfig) -> Self {
        let completion_client = build_provider(&config.provider, &config.api_key, config.base_url.as_deref());
        let embedding_client = build_embedding_client(config);

        info!(
            "LLM provider: {} (model: {}), Embedding provider: {} (model: {})",
            config.provider,
            config.model,
            config.embedding_provider.as_deref().unwrap_or(&config.provider),
            config.embedding_model,
        );

        Self {
            completion_client,
            embedding_client,
            default_model: config.model.clone(),
            embedding_model: config.embedding_model.clone(),
        }
    }

    pub fn build_agent(&self, preamble: Option<&str>) -> AnyAgent {
        let model = &self.default_model;
        match &self.completion_client {
            ProviderClient::Gemini(c) => {
                let mut b = c.agent(model);
                if let Some(p) = preamble {
                    b = b.preamble(p);
                }
                AnyAgent::Gemini(b.build())
            }
            ProviderClient::OpenAi(c) => {
                let mut b = c.agent(model);
                if let Some(p) = preamble {
                    b = b.preamble(p);
                }
                AnyAgent::OpenAi(b.build())
            }
            ProviderClient::Anthropic(c) => {
                let mut b = c.agent(model);
                if let Some(p) = preamble {
                    b = b.preamble(p);
                }
                AnyAgent::Anthropic(b.build())
            }
            ProviderClient::DeepSeek(c) => {
                let mut b = c.agent(model);
                if let Some(p) = preamble {
                    b = b.preamble(p);
                }
                AnyAgent::DeepSeek(b.build())
            }
        }
    }

    pub fn embedding_model(&self) -> AnyEmbeddingModel {
        let model = &self.embedding_model;
        match &self.embedding_client {
            EmbeddingClient::Gemini(c) => {
                use rig::client::EmbeddingsClient;
                AnyEmbeddingModel::Gemini(c.embedding_model(model))
            }
            EmbeddingClient::OpenAi(c) => {
                use rig::client::EmbeddingsClient;
                AnyEmbeddingModel::OpenAi(c.embedding_model(model))
            }
        }
    }
}

fn build_provider(provider: &str, api_key: &str, base_url: Option<&str>) -> ProviderClient {
    match provider {
        "gemini" => {
            let client = gemini::Client::new(api_key)
                .expect("Failed to initialize Gemini client");
            ProviderClient::Gemini(client)
        }
        "openai" | "openai_compatible" => {
            let mut builder = openai::Client::builder().api_key(api_key);
            if let Some(url) = base_url {
                builder = builder.base_url(url);
            }
            let client = builder.build().expect("Failed to initialize OpenAI client");
            ProviderClient::OpenAi(client.completions_api())
        }
        "anthropic" => {
            let client = anthropic::Client::new(api_key)
                .expect("Failed to initialize Anthropic client");
            ProviderClient::Anthropic(client)
        }
        "deepseek" => {
            let client = deepseek::Client::new(api_key)
                .expect("Failed to initialize DeepSeek client");
            ProviderClient::DeepSeek(client)
        }
        other => {
            warn!(
                "Unknown LLM provider '{}', falling back to openai_compatible",
                other
            );
            let mut builder = openai::Client::builder().api_key(api_key);
            if let Some(url) = base_url {
                builder = builder.base_url(url);
            }
            let client = builder
                .build()
                .expect("Failed to initialize OpenAI-compatible client");
            ProviderClient::OpenAi(client.completions_api())
        }
    }
}

// ─── Embedding support ───

pub enum EmbeddingClient {
    Gemini(gemini::Client),
    OpenAi(openai::Client),
}

pub enum AnyEmbeddingModel {
    Gemini(gemini::embedding::EmbeddingModel),
    OpenAi(openai::EmbeddingModel),
}

impl AnyEmbeddingModel {
    pub async fn embed_text(
        &self,
        text: &str,
    ) -> Result<rig::embeddings::Embedding, rig::embeddings::EmbeddingError> {
        use rig::embeddings::EmbeddingModel;
        match self {
            Self::Gemini(m) => m.embed_text(text).await,
            Self::OpenAi(m) => m.embed_text(text).await,
        }
    }
}

fn build_embedding_client(config: &LlmConfig) -> EmbeddingClient {
    let provider = config
        .embedding_provider
        .as_deref()
        .unwrap_or(&config.provider);
    let api_key = config
        .embedding_api_key
        .as_deref()
        .unwrap_or(&config.api_key);
    let base_url = config
        .embedding_base_url
        .as_deref()
        .or(config.base_url.as_deref());

    match provider {
        "gemini" => {
            let client = gemini::Client::new(api_key)
                .expect("Failed to initialize Gemini embedding client");
            EmbeddingClient::Gemini(client)
        }
        "openai" | "openai_compatible" | "deepseek" => {
            let mut builder = openai::Client::builder().api_key(api_key);
            if let Some(url) = base_url {
                builder = builder.base_url(url);
            }
            let client = builder
                .build()
                .expect("Failed to initialize OpenAI embedding client");
            EmbeddingClient::OpenAi(client)
        }
        _ => {
            let mut builder = openai::Client::builder().api_key(api_key);
            if let Some(url) = base_url {
                builder = builder.base_url(url);
            }
            let client = builder
                .build()
                .expect("Failed to initialize OpenAI-compatible embedding client");
            EmbeddingClient::OpenAi(client)
        }
    }
}
