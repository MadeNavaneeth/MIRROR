use crate::cost::TokenUsage;
use crate::providers::traits::{
    ChatMessage, ChatRequest as ProviderChatRequest, ChatResponse as ProviderChatResponse,
    Provider, ToolCall as ProviderToolCall,
};
use crate::providers::{ChatResponseChunk, ChatStream};
use crate::tools::ToolSpec;
use async_stream::try_stream;
use async_trait::async_trait;
use futures::StreamExt;
use parking_lot::Mutex;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenRouterProvider {
    api_key: Option<String>,
    client: Client,
    prices: std::collections::HashMap<String, crate::config::ModelPricing>,
    fallback_models: Vec<String>,
    proactive_credit: Option<crate::config::OpenRouterProactiveCreditConfig>,
    /// Cached `GET /api/v1/auth/key` result for proactive switching (avoid polling every request).
    credit_poll_cache: Mutex<Option<(std::time::Instant, OpenRouterCredits)>>,
}

#[derive(Debug, Serialize, Clone)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ApiChatResponse {
    choices: Vec<Choice>,
    usage: Option<ApiUsage>,
}

#[derive(Debug, Deserialize, Clone)]
struct ApiUsage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize, Clone)]
struct ResponseMessage {
    content: String,
}

#[derive(Debug, Serialize, Clone)]
struct NativeChatRequest {
    model: String,
    messages: Vec<NativeMessage>,
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<NativeToolSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
struct NativeMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<NativeToolCall>>,
}

#[derive(Debug, Serialize, Clone)]
struct NativeToolSpec {
    #[serde(rename = "type")]
    kind: String,
    function: NativeToolFunctionSpec,
}

#[derive(Debug, Serialize, Clone)]
struct NativeToolFunctionSpec {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NativeToolCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    function: NativeFunctionCall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NativeFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize, Clone)]
struct NativeChatResponse {
    choices: Vec<NativeChoice>,
    usage: Option<ApiUsage>,
}

#[derive(Debug, Deserialize, Clone)]
struct NativeChoice {
    message: NativeResponseMessage,
}

#[derive(Debug, Deserialize, Clone)]
struct NativeResponseMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<NativeToolCall>>,
}

#[derive(Debug, Deserialize, Clone)]
struct ApiStreamResponse {
    choices: Vec<StreamChoice>,
    usage: Option<ApiUsage>,
}

#[derive(Debug, Deserialize, Clone)]
struct StreamChoice {
    delta: StreamDelta,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct StreamDelta {
    content: Option<String>,
    // tool_calls omitted for now in simple streaming
}

/// Account-level credit summary from OpenRouter (`GET /api/v1/auth/key`).
/// This reflects **USD credits / limits** on your API key, not per-model token allowances from the catalog.
#[derive(Debug, Clone)]
pub struct OpenRouterCredits {
    pub usage_usd: Option<f64>,
    pub limit_usd: Option<f64>,
    pub limit_remaining_usd: Option<f64>,
    pub is_free_tier: Option<bool>,
}

impl OpenRouterCredits {
    /// Human-readable one-line summary for `mirror status`.
    #[must_use]
    pub fn summary_line(&self) -> String {
        let mut parts = Vec::new();
        if let Some(u) = self.usage_usd {
            parts.push(format!("usage ${u:.4}"));
        }
        match (self.limit_remaining_usd, self.limit_usd) {
            (Some(rem), Some(lim)) => parts.push(format!("remaining ${rem:.4} / limit ${lim:.4}")),
            (Some(rem), None) => parts.push(format!("remaining ${rem:.4}")),
            (None, Some(lim)) => parts.push(format!("limit ${lim:.4}")),
            (None, None) => {}
        }
        if let Some(ft) = self.is_free_tier {
            parts.push(format!("free_tier={ft}"));
        }
        if parts.is_empty() {
            "(no usage fields in key response)".into()
        } else {
            parts.join(" · ")
        }
    }
}

fn credits_from_key_json(root: &serde_json::Value) -> OpenRouterCredits {
    let obj = root
        .get("data")
        .filter(|v| !v.is_null())
        .unwrap_or(root);
    let num = |k: &str| obj.get(k).and_then(serde_json::Value::as_f64);
    OpenRouterCredits {
        usage_usd: num("usage"),
        limit_usd: num("limit"),
        limit_remaining_usd: num("limit_remaining"),
        is_free_tier: obj
            .get("is_free_tier")
            .and_then(|v| v.as_bool()),
    }
}

/// Fetch OpenRouter key usage and credit limits (USD). Uses the same endpoint as provider warmup.
pub async fn fetch_openrouter_credits(
    client: &Client,
    api_key: &str,
) -> anyhow::Result<OpenRouterCredits> {
    let response = client
        .get("https://openrouter.ai/api/v1/auth/key")
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        let sanitized = crate::providers::sanitize_api_error(&text);
        anyhow::bail!("OpenRouter key info ({status}): {sanitized}");
    }
    let value: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({}));
    Ok(credits_from_key_json(&value))
}

/// Whether to try `fallback_models` after an HTTP error (quota, credits, payment required).
fn openrouter_http_should_fallback(status: reqwest::StatusCode, sanitized_body: &str) -> bool {
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return true;
    }
    if status == reqwest::StatusCode::PAYMENT_REQUIRED || status.as_u16() == 402 {
        return true;
    }
    let lower = sanitized_body.to_lowercase();
    lower.contains("quota_exceeded")
        || lower.contains("insufficient credits")
        || lower.contains("insufficient credit")
        || lower.contains("out of credits")
        || lower.contains("credit balance")
        || (lower.contains("credit") && lower.contains("limit"))
}

impl OpenRouterProvider {
    pub fn new(
        api_key: Option<&str>,
        prices: std::collections::HashMap<String, crate::config::ModelPricing>,
        fallback_models: Vec<String>,
        proactive_credit: Option<crate::config::OpenRouterProactiveCreditConfig>,
    ) -> Self {
        Self {
            api_key: api_key.map(ToString::to_string),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            prices,
            fallback_models,
            proactive_credit,
            credit_poll_cache: Mutex::new(None),
        }
    }

    /// If proactive credit guard is enabled and remaining USD credits are below the threshold,
    /// return the first suitable model from `fallback_models`; otherwise return `requested`.
    async fn resolve_openrouter_model(&self, requested: &str) -> String {
        let Some(cfg) = self.proactive_credit else {
            return requested.to_string();
        };
        let Some(api_key) = self.api_key.as_ref() else {
            return requested.to_string();
        };
        if self.fallback_models.is_empty() {
            tracing::warn!(
                "OpenRouter proactive credit threshold is set but fallback_models is empty; keeping requested model"
            );
            return requested.to_string();
        }

        let poll_interval =
            std::time::Duration::from_secs(cfg.poll_secs.max(10));
        let now = std::time::Instant::now();

        let cached = {
            let guard = self.credit_poll_cache.lock();
            guard.as_ref().and_then(|(t, c)| {
                if now.duration_since(*t) < poll_interval {
                    Some(c.clone())
                } else {
                    None
                }
            })
        };

        let credits = if let Some(c) = cached {
            c
        } else {
            match fetch_openrouter_credits(&self.client, api_key).await {
                Ok(c) => {
                    let mut guard = self.credit_poll_cache.lock();
                    *guard = Some((now, c.clone()));
                    c
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "OpenRouter proactive credit fetch failed; using requested model"
                    );
                    return requested.to_string();
                }
            }
        };

        let Some(rem) = credits.limit_remaining_usd else {
            return requested.to_string();
        };

        if rem >= cfg.threshold_usd {
            return requested.to_string();
        }

        let replacement = self
            .fallback_models
            .iter()
            .find(|m| *m != requested)
            .unwrap_or_else(|| &self.fallback_models[0]);

        if replacement == requested {
            return requested.to_string();
        }

        tracing::info!(
            remaining_usd = rem,
            threshold_usd = cfg.threshold_usd,
            requested,
            replacement,
            "OpenRouter proactive: switching model due to low account credits (USD)"
        );
        replacement.clone()
    }

    fn convert_tools(tools: Option<&[ToolSpec]>) -> Option<Vec<NativeToolSpec>> {
        let items = tools?;
        if items.is_empty() {
            return None;
        }
        Some(
            items
                .iter()
                .map(|tool| NativeToolSpec {
                    kind: "function".to_string(),
                    function: NativeToolFunctionSpec {
                        name: tool.name.clone(),
                        description: tool.description.clone(),
                        parameters: tool.parameters.clone(),
                    },
                })
                .collect(),
        )
    }

    fn convert_messages(messages: &[ChatMessage]) -> Vec<NativeMessage> {
        messages
            .iter()
            .map(|m| {
                if m.role == "assistant" {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&m.content) {
                        if let Some(tool_calls_value) = value.get("tool_calls") {
                            if let Ok(parsed_calls) =
                                serde_json::from_value::<Vec<ProviderToolCall>>(
                                    tool_calls_value.clone(),
                                )
                            {
                                let tool_calls = parsed_calls
                                    .into_iter()
                                    .map(|tc| NativeToolCall {
                                        id: Some(tc.id),
                                        kind: Some("function".to_string()),
                                        function: NativeFunctionCall {
                                            name: tc.name,
                                            arguments: tc.arguments,
                                        },
                                    })
                                    .collect::<Vec<_>>();
                                let content = value
                                    .get("content")
                                    .and_then(serde_json::Value::as_str)
                                    .map(ToString::to_string);
                                return NativeMessage {
                                    role: "assistant".to_string(),
                                    content,
                                    tool_call_id: None,
                                    tool_calls: Some(tool_calls),
                                };
                            }
                        }
                    }
                }

                if m.role == "tool" {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&m.content) {
                        let tool_call_id = value
                            .get("tool_call_id")
                            .and_then(serde_json::Value::as_str)
                            .map(ToString::to_string);
                        let content = value
                            .get("content")
                            .and_then(serde_json::Value::as_str)
                            .map(ToString::to_string);
                        return NativeMessage {
                            role: "tool".to_string(),
                            content,
                            tool_call_id,
                            tool_calls: None,
                        };
                    }
                }

                NativeMessage {
                    role: m.role.clone(),
                    content: Some(m.content.clone()),
                    tool_call_id: None,
                    tool_calls: None,
                }
            })
            .collect()
    }

    fn parse_native_response(
        &self,
        message: NativeResponseMessage,
        usage: Option<ApiUsage>,
        model: &str,
    ) -> ProviderChatResponse {
        let tool_calls = message
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|tc| ProviderToolCall {
                id: tc.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                name: tc.function.name,
                arguments: tc.function.arguments,
            })
            .collect::<Vec<_>>();

        let usage = usage.map(|u| {
            let pricing = self.prices.get(model);
            let input_cost = pricing.map(|p| p.input).unwrap_or(0.0);
            let output_cost = pricing.map(|p| p.output).unwrap_or(0.0);
            TokenUsage::new(
                model,
                u.prompt_tokens,
                u.completion_tokens,
                input_cost,
                output_cost,
            )
        });

        ProviderChatResponse {
            text: message.content,
            tool_calls,
            usage,
        }
    }
}

#[async_trait]
impl Provider for OpenRouterProvider {
    async fn warmup(&self) -> anyhow::Result<()> {
        // Hit a lightweight endpoint to establish TLS + HTTP/2 connection pool.
        // This prevents the first real chat request from timing out on cold start.
        if let Some(api_key) = self.api_key.as_ref() {
            self.client
                .get("https://openrouter.ai/api/v1/auth/key")
                .header("Authorization", format!("Bearer {api_key}"))
                .send()
                .await?
                .error_for_status()?;
        }
        Ok(())
    }

    async fn chat_with_system(
        &self,
        system_prompt: Option<&str>,
        message: &str,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<String> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenRouter API key not set. Run `mirror onboard` or set OPENROUTER_API_KEY env var."))?;

        let effective_model = self.resolve_openrouter_model(model).await;

        let mut messages = Vec::new();

        if let Some(sys) = system_prompt {
            messages.push(Message {
                role: "system".to_string(),
                content: sys.to_string(),
            });
        }

        messages.push(Message {
            role: "user".to_string(),
            content: message.to_string(),
        });

        let request = ChatRequest {
            model: effective_model.clone(),
            messages,
            temperature,
            stream: None,
        };

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {api_key}"))
            .header(
                "HTTP-Referer",
                "https://github.com/theonlyhennygod/mirror",
            )
            .header("X-Title", "Mirror")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            let sanitized = super::sanitize_api_error(&body);

            if openrouter_http_should_fallback(status, &sanitized) {
                for fallback in &self.fallback_models {
                    tracing::info!(
                        "OpenRouter primary model {} failed (retryable). Trying fallback: {}",
                        effective_model,
                        fallback
                    );

                    let fallback_request = ChatRequest {
                        model: fallback.clone(),
                        messages: vec![
                            Message {
                                role: "system".into(),
                                content: system_prompt.unwrap_or("").into(),
                            },
                            Message {
                                role: "user".into(),
                                content: message.into(),
                            },
                        ],
                        temperature,
                        stream: None,
                    };

                    let fallback_response = self
                        .client
                        .post("https://openrouter.ai/api/v1/chat/completions")
                        .header("Authorization", format!("Bearer {api_key}"))
                        .header("HTTP-Referer", "https://github.com/theonlyhennygod/mirror")
                        .header("X-Title", "Mirror")
                        .json(&fallback_request)
                        .send()
                        .await?;

                    if fallback_response.status().is_success() {
                        let chat_response: ApiChatResponse = fallback_response.json().await?;
                        return chat_response
                            .choices
                            .into_iter()
                            .next()
                            .map(|c| c.message.content)
                            .ok_or_else(|| anyhow::anyhow!("No response from OpenRouter fallback"));
                    }
                }
            }
            return Err(anyhow::anyhow!(
                "OpenRouter API error ({status}): {sanitized}"
            ));
        }

        let chat_response: ApiChatResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("OpenRouter response JSON: {e}"))?;

        chat_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("No response from OpenRouter"))
    }

    async fn chat_with_history(
        &self,
        messages: &[ChatMessage],
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<String> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenRouter API key not set. Run `mirror onboard` or set OPENROUTER_API_KEY env var."))?;

        let effective_model = self.resolve_openrouter_model(model).await;

        let api_messages: Vec<Message> = messages
            .iter()
            .map(|m| Message {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = ChatRequest {
            model: effective_model.clone(),
            messages: api_messages.clone(),
            temperature,
            stream: None,
        };

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {api_key}"))
            .header(
                "HTTP-Referer",
                "https://github.com/theonlyhennygod/mirror",
            )
            .header("X-Title", "Mirror")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            let sanitized = super::sanitize_api_error(&body);

            if openrouter_http_should_fallback(status, &sanitized) {
                for fallback in &self.fallback_models {
                    tracing::info!(
                        "OpenRouter primary model {} failed (retryable). Trying fallback: {}",
                        effective_model,
                        fallback
                    );

                    let fallback_request = ChatRequest {
                        model: fallback.clone(),
                        messages: api_messages.clone(),
                        temperature,
                        stream: None,
                    };

                    let fallback_response = self
                        .client
                        .post("https://openrouter.ai/api/v1/chat/completions")
                        .header("Authorization", format!("Bearer {api_key}"))
                        .header("HTTP-Referer", "https://github.com/theonlyhennygod/mirror")
                        .header("X-Title", "Mirror")
                        .json(&fallback_request)
                        .send()
                        .await?;

                    if fallback_response.status().is_success() {
                        let chat_response: ApiChatResponse = fallback_response.json().await?;
                        return chat_response
                            .choices
                            .into_iter()
                            .next()
                            .map(|c| c.message.content)
                            .ok_or_else(|| anyhow::anyhow!("No response from OpenRouter fallback"));
                    }
                }
            }
            return Err(anyhow::anyhow!(
                "OpenRouter API error ({status}): {sanitized}"
            ));
        }

        let chat_response: ApiChatResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("OpenRouter response JSON: {e}"))?;

        chat_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("No response from OpenRouter"))
    }

    async fn chat(
        &self,
        request: ProviderChatRequest<'_>,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<ProviderChatResponse> {
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
            "OpenRouter API key not set. Run `mirror onboard` or set OPENROUTER_API_KEY env var."
        )
        })?;

        let effective_model = self.resolve_openrouter_model(model).await;

        let tools = Self::convert_tools(request.tools);
        let native_request = NativeChatRequest {
            model: effective_model.clone(),
            messages: Self::convert_messages(request.messages),
            temperature,
            stream: None,
            tool_choice: tools.as_ref().map(|_| "auto".to_string()),
            tools: tools.clone(),
        };

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {api_key}"))
            .header(
                "HTTP-Referer",
                "https://github.com/theonlyhennygod/mirror",
            )
            .header("X-Title", "Mirror")
            .json(&native_request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            let sanitized = super::sanitize_api_error(&body);
            if openrouter_http_should_fallback(status, &sanitized) {
                for fallback in &self.fallback_models {
                    tracing::info!(
                        "OpenRouter primary model {} failed (retryable). Trying fallback: {}",
                        effective_model,
                        fallback
                    );
                    let native_request_fallback = NativeChatRequest {
                        model: fallback.clone(),
                        messages: Self::convert_messages(request.messages),
                        temperature,
                        stream: None,
                        tool_choice: tools.as_ref().map(|_| "auto".to_string()),
                        tools: Self::convert_tools(request.tools), // re-convert since native_request moved it
                    };
                    let fallback_response = self
                        .client
                        .post("https://openrouter.ai/api/v1/chat/completions")
                        .header("Authorization", format!("Bearer {api_key}"))
                        .header("HTTP-Referer", "https://github.com/theonlyhennygod/mirror")
                        .header("X-Title", "Mirror")
                        .json(&native_request_fallback)
                        .send()
                        .await?;

                    if fallback_response.status().is_success() {
                        let native_response: NativeChatResponse = fallback_response.json().await?;
                        let usage = native_response.usage.clone();
                        let message = native_response
                            .choices
                            .into_iter()
                            .next()
                            .map(|c| c.message)
                            .ok_or_else(|| anyhow::anyhow!("No response from OpenRouter fallback"))?;
                        return Ok(self.parse_native_response(message, usage, fallback));
                    }
                }
            }
            return Err(anyhow::anyhow!(
                "OpenRouter API error ({status}): {sanitized}"
            ));
        }

        let native_response: NativeChatResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("OpenRouter response JSON: {e}"))?;
        let usage = native_response.usage.clone();
        let message = native_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message)
            .ok_or_else(|| anyhow::anyhow!("No response from OpenRouter"))?;
        Ok(self.parse_native_response(message, usage, effective_model.as_str()))
    }

    async fn chat_stream(
        &self,
        request: ProviderChatRequest<'_>,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<ChatStream> {
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
            "OpenRouter API key not set. Run `mirror onboard` or set OPENROUTER_API_KEY env var."
        )
        })?;

        let effective_model = self.resolve_openrouter_model(model).await;

        let tools = Self::convert_tools(request.tools);
        let native_request = NativeChatRequest {
            model: effective_model.clone(),
            messages: Self::convert_messages(request.messages),
            temperature,
            tool_choice: tools.as_ref().map(|_| "auto".to_string()),
            tools,
            stream: Some(true),
        };

        let client = self.client.clone();
        let url = "https://openrouter.ai/api/v1/chat/completions".to_string();
        let api_key = api_key.clone();
        let model_str = effective_model;
        let fallbacks = self.fallback_models.clone();
        let prices = self.prices.clone();

        let stream = try_stream! {
            let mut stream_response = client
                .post(&url)
                .header("Authorization", format!("Bearer {api_key}"))
                .header("HTTP-Referer", "https://github.com/theonlyhennygod/mirror")
                .header("X-Title", "Mirror")
                .json(&native_request)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Request failed: {e}"))?;

            let mut active_model = model_str.clone();

            if !stream_response.status().is_success() {
                let status = stream_response.status();
                let err_body = stream_response.text().await.unwrap_or_default();
                let sanitized = crate::providers::sanitize_api_error(&err_body);
                if openrouter_http_should_fallback(status, &sanitized) {
                    let mut picked: Option<reqwest::Response> = None;
                    for fallback in &fallbacks {
                        tracing::info!(
                            "OpenRouter primary model {} failed (stream, retryable). Trying fallback: {}",
                            model_str,
                            fallback
                        );
                        let mut fallback_request = native_request.clone();
                        fallback_request.model = fallback.clone();

                        let fallback_res = client
                            .post(&url)
                            .header("Authorization", format!("Bearer {api_key}"))
                            .header("HTTP-Referer", "https://github.com/theonlyhennygod/mirror")
                            .header("X-Title", "Mirror")
                            .json(&fallback_request)
                            .send()
                            .await;

                        if let Ok(res) = fallback_res {
                            if res.status().is_success() {
                                picked = Some(res);
                                active_model = fallback.clone();
                                break;
                            }
                        }
                    }
                    stream_response = picked.ok_or_else(|| {
                        anyhow::anyhow!("OpenRouter API error ({status}): {sanitized}")
                    })?;
                } else {
                    Err(anyhow::anyhow!(
                        "OpenRouter API error ({status}): {sanitized}"
                    ))?;
                    return;
                }
            }

            let model_id = active_model;
            let mut bytes_stream = stream_response.bytes_stream();
            let mut buffer = Vec::new();

            while let Some(chunk_res) = bytes_stream.next().await {
                let chunk = chunk_res.map_err(|e| anyhow::anyhow!("Stream error: {e}"))?;
                buffer.extend_from_slice(&chunk);

                while let Some(i) = buffer.iter().position(|&b| b == b'\n') {
                    let line = buffer.drain(..=i).collect::<Vec<u8>>();
                    let line_str = String::from_utf8_lossy(&line);
                    let line_str = line_str.trim();

                    if line_str.is_empty() { continue; }
                    if !line_str.starts_with("data: ") { continue; }

                    let data = &line_str["data: ".len()..];
                    if data == "[DONE]" {
                        break;
                    }

                    if let Ok(stream_res) = serde_json::from_str::<ApiStreamResponse>(data) {
                        if let Some(choice) = stream_res.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                if !content.is_empty() {
                                    yield ChatResponseChunk::Content(content.clone());
                                }
                            }
                        }
                        if let Some(usage) = stream_res.usage {
                            let pricing = prices.get(&model_id);
                            let input_cost = pricing.map(|p| p.input).unwrap_or(0.0);
                            let output_cost = pricing.map(|p| p.output).unwrap_or(0.0);

                            yield ChatResponseChunk::Usage(TokenUsage::new(
                                &model_id,
                                usage.prompt_tokens,
                                usage.completion_tokens,
                                input_cost,
                                output_cost,
                            ));
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    fn supports_native_tools(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::traits::{ChatMessage, Provider};

    #[test]
    fn creates_with_key() {
        let provider = OpenRouterProvider::new(Some("sk-or-123"), std::collections::HashMap::new(), vec![], None);
        assert_eq!(provider.api_key.as_deref(), Some("sk-or-123"));
    }

    #[test]
    fn creates_without_key() {
        let provider = OpenRouterProvider::new(None, std::collections::HashMap::new(), vec![], None);
        assert!(provider.api_key.is_none());
    }

    #[tokio::test]
    async fn warmup_without_key_is_noop() {
        let provider = OpenRouterProvider::new(None, std::collections::HashMap::new(), vec![], None);
        let result = provider.warmup().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_with_system_fails_without_key() {
        let provider = OpenRouterProvider::new(None, std::collections::HashMap::new(), vec![], None);
        let result = provider
            .chat_with_system(Some("system"), "hello", "openai/gpt-4o", 0.2)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key not set"));
    }

    #[tokio::test]
    async fn chat_with_history_fails_without_key() {
        let provider = OpenRouterProvider::new(None, std::collections::HashMap::new(), vec![], None);
        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: "be concise".into(),
            },
            ChatMessage {
                role: "user".into(),
                content: "hello".into(),
            },
        ];

        let result = provider
            .chat_with_history(&messages, "anthropic/claude-sonnet-4", 0.7)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key not set"));
    }

    #[test]
    fn chat_request_serializes_with_system_and_user() {
        let request = ChatRequest {
            model: "anthropic/claude-sonnet-4".into(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: "You are helpful".into(),
                },
                Message {
                    role: "user".into(),
                    content: "Summarize this".into(),
                },
            ],
            temperature: 0.5,
            stream: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("anthropic/claude-sonnet-4"));
        assert!(json.contains("\"role\":\"system\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"temperature\":0.5"));
    }

    #[test]
    fn chat_request_serializes_history_messages() {
        let messages = [
            ChatMessage {
                role: "assistant".into(),
                content: "Previous answer".into(),
            },
            ChatMessage {
                role: "user".into(),
                content: "Follow-up".into(),
            },
        ];

        let request = ChatRequest {
            model: "google/gemini-2.5-pro".into(),
            messages: messages
                .iter()
                .map(|msg| Message {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                })
                .collect(),
            temperature: 0.7,
            stream: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"role\":\"assistant\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("google/gemini-2.5-pro"));
    }

    #[test]
    fn response_deserializes_single_choice() {
        let json = r#"{"choices":[{"message":{"content":"Hi from OpenRouter"}}]}"#;

        let response: ApiChatResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.content, "Hi from OpenRouter");
    }

    #[test]
    fn response_deserializes_empty_choices() {
        let json = r#"{"choices":[]}"#;

        let response: ApiChatResponse = serde_json::from_str(json).unwrap();

        assert!(response.choices.is_empty());
    }

    #[test]
    fn openrouter_fallback_matches_credit_errors() {
        assert!(super::openrouter_http_should_fallback(
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            ""
        ));
        assert!(super::openrouter_http_should_fallback(
            reqwest::StatusCode::PAYMENT_REQUIRED,
            ""
        ));
        assert!(super::openrouter_http_should_fallback(
            reqwest::StatusCode::OK,
            "{\"error\":{\"message\":\"quota_exceeded\"}}"
        ));
        assert!(super::openrouter_http_should_fallback(
            reqwest::StatusCode::BAD_REQUEST,
            "Insufficient credits for model xyz"
        ));
        assert!(!super::openrouter_http_should_fallback(
            reqwest::StatusCode::BAD_REQUEST,
            "invalid json schema"
        ));
    }
}
