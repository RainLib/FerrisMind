use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub surreal: SurrealConfig,
    pub jwt: JwtConfig,
    pub llm: LlmConfig,
    pub upload: UploadConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SurrealConfig {
    pub addr: String,
    pub user: String,
    pub pass: String,
    pub ns: String,
    pub db: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub api_key: String,
    pub api_base: String,
    pub model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadConfig {
    pub dir: String,
    pub max_file_size_mb: u64,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
            },
            surreal: SurrealConfig {
                addr: std::env::var("SURREAL_ADDR")
                    .unwrap_or_else(|_| "ws://127.0.0.1:8000".to_string()),
                user: std::env::var("SURREAL_USER").unwrap_or_else(|_| "root".to_string()),
                pass: std::env::var("SURREAL_PASS").unwrap_or_else(|_| "root".to_string()),
                ns: std::env::var("SURREAL_NS").unwrap_or_else(|_| "notebook".to_string()),
                db: std::env::var("SURREAL_DB").unwrap_or_else(|_| "main".to_string()),
            },
            jwt: JwtConfig {
                secret: std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "dev-secret-change-me".to_string()),
                expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()?,
            },
            llm: LlmConfig {
                api_key: std::env::var("LLM_API_KEY").unwrap_or_default(),
                api_base: std::env::var("LLM_API_BASE")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
                model: std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
                max_tokens: std::env::var("LLM_MAX_TOKENS")
                    .unwrap_or_else(|_| "4096".to_string())
                    .parse()?,
            },
            upload: UploadConfig {
                dir: std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()),
                max_file_size_mb: std::env::var("MAX_FILE_SIZE_MB")
                    .unwrap_or_else(|_| "50".to_string())
                    .parse()?,
            },
        })
    }
}
