// Provider API key validation via HTTP
// Ported from ipc-handlers.ts validateApiKeyWithProvider()

use reqwest::Client;
use serde_json::{json, Value};

/// Validate an API key by making a test request to the provider
pub async fn validate_api_key(
    provider_type: &str,
    api_key: &str,
    base_url: Option<&str>,
) -> Result<Value, String> {
    let profile = get_validation_profile(provider_type);
    if profile == "none" {
        return Ok(json!({ "valid": true }));
    }

    let trimmed_key = api_key.trim();
    if trimmed_key.is_empty() {
        return Ok(json!({ "valid": false, "error": "API key is required" }));
    }

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    match profile {
        "openai-compatible" => validate_openai_compatible(&client, provider_type, trimmed_key, base_url).await,
        "google-query-key" => validate_google_query_key(&client, provider_type, trimmed_key, base_url).await,
        "anthropic-header" => validate_anthropic_header(&client, trimmed_key, base_url).await,
        "openrouter" => validate_openrouter(&client, trimmed_key).await,
        _ => Ok(json!({ "valid": false, "error": format!("Unsupported provider: {}", provider_type) })),
    }
}

fn get_validation_profile(provider_type: &str) -> &'static str {
    match provider_type {
        "anthropic" => "anthropic-header",
        "google" => "google-query-key",
        "openrouter" => "openrouter",
        "ollama" => "none",
        _ => "openai-compatible",
    }
}

fn normalize_base_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

/// Classify HTTP response status to valid/invalid
fn classify_response(status: u16, body: &Value) -> Value {
    if (200..300).contains(&status) {
        return json!({ "valid": true });
    }
    if status == 429 {
        return json!({ "valid": true }); // rate-limited but key works
    }
    if status == 401 || status == 403 {
        return json!({ "valid": false, "error": "Invalid API key" });
    }

    let msg = body
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .or_else(|| body.get("message").and_then(|m| m.as_str()))
        .unwrap_or("Unknown API error");

    json!({ "valid": false, "error": format!("API error {}: {}", status, msg) })
}

async fn perform_request(client: &Client, url: &str, headers: &[(String, String)]) -> Result<Value, String> {
    let mut req = client.get(url);
    for (key, value) in headers {
        req = req.header(key.as_str(), value.as_str());
    }

    let response = req.send().await
        .map_err(|e| format!("Connection error: {}", e))?;

    let status = response.status().as_u16();
    let body: Value = response.json().await.unwrap_or(json!({}));

    Ok(classify_response(status, &body))
}

async fn validate_openai_compatible(
    client: &Client,
    provider_type: &str,
    api_key: &str,
    base_url: Option<&str>,
) -> Result<Value, String> {
    let base = match base_url {
        Some(url) if !url.trim().is_empty() => normalize_base_url(url),
        _ => return Ok(json!({ "valid": false, "error": format!("Base URL is required for provider \"{}\" validation", provider_type) })),
    };

    let models_url = format!("{}/models?limit=1", base);
    let headers = vec![("Authorization".to_string(), format!("Bearer {}", api_key))];

    let result = perform_request(client, &models_url, &headers).await?;

    // If /models returned 404, fallback to /chat/completions probe
    if let Some(error) = result.get("error").and_then(|e| e.as_str()) {
        if error.contains("API error: 404") || error.contains("API error 404") {
            let chat_url = format!("{}/chat/completions", base);
            return perform_chat_completions_probe(client, &chat_url, api_key).await;
        }
    }

    Ok(result)
}

async fn perform_chat_completions_probe(
    client: &Client,
    url: &str,
    api_key: &str,
) -> Result<Value, String> {
    let body = json!({
        "model": "validation-probe",
        "messages": [{ "role": "user", "content": "hi" }],
        "max_tokens": 1,
    });

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    let status = response.status().as_u16();
    let resp_body: Value = response.json().await.unwrap_or(json!({}));

    if status == 401 || status == 403 {
        return Ok(json!({ "valid": false, "error": "Invalid API key" }));
    }
    // 200, 400 (bad model but key accepted), 429 → key is valid
    if (200..300).contains(&status) || status == 400 || status == 429 {
        return Ok(json!({ "valid": true }));
    }

    Ok(classify_response(status, &resp_body))
}

async fn validate_google_query_key(
    client: &Client,
    provider_type: &str,
    api_key: &str,
    base_url: Option<&str>,
) -> Result<Value, String> {
    let base = match base_url {
        Some(url) if !url.trim().is_empty() => normalize_base_url(url),
        _ => return Ok(json!({ "valid": false, "error": format!("Base URL is required for provider \"{}\" validation", provider_type) })),
    };

    let url = format!("{}/models?pageSize=1&key={}", base, urlencoding_encode(api_key));
    perform_request(client, &url, &[]).await
}

async fn validate_anthropic_header(
    client: &Client,
    api_key: &str,
    base_url: Option<&str>,
) -> Result<Value, String> {
    let base = base_url
        .filter(|u| !u.trim().is_empty())
        .map(|u| normalize_base_url(u))
        .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string());

    let url = format!("{}/models?limit=1", base);
    let headers = vec![
        ("x-api-key".to_string(), api_key.to_string()),
        ("anthropic-version".to_string(), "2023-06-01".to_string()),
    ];

    perform_request(client, &url, &headers).await
}

async fn validate_openrouter(
    client: &Client,
    api_key: &str,
) -> Result<Value, String> {
    let url = "https://openrouter.ai/api/v1/auth/key";
    let headers = vec![("Authorization".to_string(), format!("Bearer {}", api_key))];
    perform_request(client, url, &headers).await
}

/// Simple URL encoding for query parameter values
fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
