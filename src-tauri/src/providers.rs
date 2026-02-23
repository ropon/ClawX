// Provider Registry — single source of truth for backend provider metadata
//
// Internal module providing provider model definitions and metadata.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub base_url: &'static str,
    pub api: &'static str,
    pub api_key_env: &'static str,
    pub models: Option<Vec<ProviderModel>>,
}

#[derive(Debug, Clone)]
struct ProviderMeta {
    env_var: Option<&'static str>,
    default_model: Option<&'static str>,
    config: Option<ProviderConfig>,
}

fn get_registry() -> Vec<(&'static str, ProviderMeta)> {
    vec![
        ("anthropic", ProviderMeta {
            env_var: Some("ANTHROPIC_API_KEY"),
            default_model: Some("anthropic/claude-opus-4-6"),
            config: None, // Built-in to OpenClaw's model registry
        }),
        ("openai", ProviderMeta {
            env_var: Some("OPENAI_API_KEY"),
            default_model: Some("openai/gpt-5.2"),
            config: Some(ProviderConfig {
                base_url: "https://api.openai.com/v1",
                api: "openai-responses",
                api_key_env: "OPENAI_API_KEY",
                models: None,
            }),
        }),
        ("google", ProviderMeta {
            env_var: Some("GEMINI_API_KEY"),
            default_model: Some("google/gemini-3-pro-preview"),
            config: Some(ProviderConfig {
                base_url: "https://generativelanguage.googleapis.com/v1beta",
                api: "google",
                api_key_env: "GEMINI_API_KEY",
                models: None,
            }),
        }),
        ("openrouter", ProviderMeta {
            env_var: Some("OPENROUTER_API_KEY"),
            default_model: Some("openrouter/anthropic/claude-opus-4.6"),
            config: Some(ProviderConfig {
                base_url: "https://openrouter.ai/api/v1",
                api: "openai-completions",
                api_key_env: "OPENROUTER_API_KEY",
                models: None,
            }),
        }),
        ("moonshot", ProviderMeta {
            env_var: Some("MOONSHOT_API_KEY"),
            default_model: Some("moonshot/kimi-k2.5"),
            config: Some(ProviderConfig {
                base_url: "https://api.moonshot.cn/v1",
                api: "openai-completions",
                api_key_env: "MOONSHOT_API_KEY",
                models: Some(vec![ProviderModel {
                    id: "kimi-k2.5".to_string(),
                    name: "Kimi K2.5".to_string(),
                }]),
            }),
        }),
        ("deepseek", ProviderMeta {
            env_var: Some("DEEPSEEK_API_KEY"),
            default_model: Some("deepseek/deepseek-chat"),
            config: Some(ProviderConfig {
                base_url: "https://api.deepseek.com",
                api: "openai-completions",
                api_key_env: "DEEPSEEK_API_KEY",
                models: Some(vec![
                    ProviderModel { id: "deepseek-chat".to_string(), name: "DeepSeek Chat (V3)".to_string() },
                    ProviderModel { id: "deepseek-reasoner".to_string(), name: "DeepSeek Reasoner (R1)".to_string() },
                ]),
            }),
        }),
        ("qwen", ProviderMeta {
            env_var: Some("DASHSCOPE_API_KEY"),
            default_model: Some("qwen/qwen-plus"),
            config: Some(ProviderConfig {
                base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
                api: "openai-completions",
                api_key_env: "DASHSCOPE_API_KEY",
                models: Some(vec![
                    ProviderModel { id: "qwen-plus".to_string(), name: "Qwen Plus".to_string() },
                    ProviderModel { id: "qwen-max".to_string(), name: "Qwen Max".to_string() },
                    ProviderModel { id: "qwen-turbo".to_string(), name: "Qwen Turbo".to_string() },
                ]),
            }),
        }),
        ("siliconflow", ProviderMeta {
            env_var: Some("SILICONFLOW_API_KEY"),
            default_model: Some("siliconflow/deepseek-ai/DeepSeek-V3"),
            config: Some(ProviderConfig {
                base_url: "https://api.siliconflow.cn/v1",
                api: "openai-completions",
                api_key_env: "SILICONFLOW_API_KEY",
                models: None,
            }),
        }),
        ("groq", ProviderMeta { env_var: Some("GROQ_API_KEY"), default_model: None, config: None }),
        ("deepgram", ProviderMeta { env_var: Some("DEEPGRAM_API_KEY"), default_model: None, config: None }),
        ("cerebras", ProviderMeta { env_var: Some("CEREBRAS_API_KEY"), default_model: None, config: None }),
        ("xai", ProviderMeta { env_var: Some("XAI_API_KEY"), default_model: None, config: None }),
        ("mistral", ProviderMeta { env_var: Some("MISTRAL_API_KEY"), default_model: None, config: None }),
    ]
}

pub fn get_provider_env_var(provider_type: &str) -> Option<&'static str> {
    get_registry().into_iter()
        .find(|(t, _)| *t == provider_type)
        .and_then(|(_, m)| m.env_var)
}

pub fn get_provider_default_model(provider_type: &str) -> Option<&'static str> {
    get_registry().into_iter()
        .find(|(t, _)| *t == provider_type)
        .and_then(|(_, m)| m.default_model)
}

pub fn get_provider_config(provider_type: &str) -> Option<ProviderConfig> {
    get_registry().into_iter()
        .find(|(t, _)| *t == provider_type)
        .and_then(|(_, m)| m.config)
}

#[allow(dead_code)]
pub fn get_keyable_provider_types() -> Vec<&'static str> {
    get_registry().into_iter()
        .filter(|(_, m)| m.env_var.is_some())
        .map(|(t, _)| t)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_provider_env_var() {
        assert_eq!(get_provider_env_var("anthropic"), Some("ANTHROPIC_API_KEY"));
        assert_eq!(get_provider_env_var("openai"), Some("OPENAI_API_KEY"));
        assert_eq!(get_provider_env_var("unknown"), None);
    }

    #[test]
    fn test_get_provider_default_model() {
        assert_eq!(get_provider_default_model("anthropic"), Some("anthropic/claude-opus-4-6"));
        assert_eq!(get_provider_default_model("groq"), None);
    }

    #[test]
    fn test_get_provider_config() {
        let config = get_provider_config("openai").unwrap();
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.api, "openai-responses");
        assert!(get_provider_config("anthropic").is_none());
    }

    #[test]
    fn test_keyable_providers() {
        let types = get_keyable_provider_types();
        assert!(types.contains(&"anthropic"));
        assert!(types.contains(&"openai"));
        assert!(types.len() >= 7);
    }
}
