//! API adapter for OpenRouter.
//!
//! Reference: https://openrouter.ai/docs/api-reference/overview

use std::mem;

use const_format::concatcp;

use serde::{Deserialize, Serialize};
use serde_json::Number;

use reqwest::header::CONTENT_TYPE;
use reqwest::Client;

use crate::apis::ApiClient as GenericApiClient;
use crate::utils::error::{ApiKeyCheckError, ApiMakeCallError};

/// OpenRouter API request URL prefix.
const OPENRT_API_PREFIX: &str = "https://openrouter.ai/api/v1";

/// OpenRouter model choice. Not using `openrouter/auto` to auto select because
/// sometimes it would pick a deep reasoning model that would likely disregard
/// the structured JSON output instructions.
const OPENRT_MODEL_NAME: &str = "mistralai/mistral-large";

/// API key validity check request URL.
/// Accompolished with the rate/credit limit checking API.
const CHECK_API_KEY_URL: &str = concatcp!(OPENRT_API_PREFIX, "/auth/key");

/// API chat completion request URL.
const CHAT_COMPLETION_URL: &str = concatcp!(OPENRT_API_PREFIX, "/chat/completions");

/// Default OpenRouter API key with no credits (only free quota access).
const FREE_QUOTA_API_KEY: &str =
    "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";

/// Max output tokens cap.
const MAX_OUTPUT_TOKENS: u32 = 500;

/// OpenRouter API client.
pub(crate) struct ApiClient {
    api_key: String,
    client: Client,
}

/// OpenRouter API validation response body.
#[derive(Serialize, Deserialize, Debug)]
struct ApiKeyCheckResponse {
    data: ApiKeyCheckResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiKeyCheckResponseData {
    label: String,
    limit: Option<Number>,
    usage: Number,
}

/// OpenRouter detection API call response body.
#[derive(Serialize, Deserialize, Debug)]
struct ApiDetectionResponse {
    id: String,
    model: String,
    choices: Vec<ApiDetectionResponseChoice>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiDetectionResponseChoice {
    message: ApiDetectionResponseMessage,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiDetectionResponseMessage {
    content: String,
}

impl ApiClient {
    /// Creates a new OpenRouter API client. Only successful if passes the API key validity check.
    /// Uses the default free quota API KEY if input key is `None`.
    pub(crate) async fn new(api_key: Option<String>) -> Result<Self, ApiKeyCheckError> {
        let client = Self {
            api_key: api_key.unwrap_or(FREE_QUOTA_API_KEY.into()),
            client: Client::new(),
        };

        client.check_api_key().await?;
        Ok(client)
    }

    /// Makes an API key validity check request and returns an error if unsuccessful.
    async fn check_api_key(&self) -> Result<(), ApiKeyCheckError> {
        log::debug!("Choosing the OpenRouter API...");

        let response = self
            .client
            .get(CHECK_API_KEY_URL)
            .bearer_auth(self.api_key.clone())
            .send()
            .await?;

        if !response.status().is_success() {
            // probably network error or authorization failure
            let status = response.status();
            let text = response.text().await?;
            return Err(ApiKeyCheckError::status(format!(
                "API key validation failed with {}: {}",
                status, text
            )));
        } else {
            // successful (quota not guaranteed)
            let resp_data = response.json::<ApiKeyCheckResponse>().await?.data;
            if let Some(limit) = resp_data.limit {
                if !limit.is_f64() {
                    return Err(ApiKeyCheckError::limit(format!(
                        "API key validation successful, but invalid limit '{}'",
                        limit
                    )));
                }
                if !resp_data.usage.is_f64() {
                    return Err(ApiKeyCheckError::limit(format!(
                        "API key validation successful, but invalid usage '{}'",
                        resp_data.usage
                    )));
                }
                if resp_data.usage.as_f64().unwrap() >= limit.as_f64().unwrap() {
                    return Err(ApiKeyCheckError::limit(
                        "API key validation successful, but credit used up",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Makes an detection API call and returns the response.
    pub(crate) async fn call(&self, prompt: String) -> Result<(u8, String), ApiMakeCallError> {
        log::debug!("Making API call to OpenRouter...");

        let request = serde_json::json!({
            "model": OPENRT_MODEL_NAME,
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "max_tokens": MAX_OUTPUT_TOKENS,
        });

        let response = self
            .client
            .post(CHAT_COMPLETION_URL)
            .bearer_auth(self.api_key.clone())
            .header(CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            // probably network error or rate limited
            let status = response.status();
            let text = response.text().await?;
            Err(ApiMakeCallError::status(format!(
                "API call failed with {}: {}",
                status, text
            )))
        } else {
            // successful
            let mut resp = response.json::<ApiDetectionResponse>().await?;
            if resp.choices.is_empty() {
                return Err(ApiMakeCallError::parse("no choices found in response"));
            }
            let output = mem::take(&mut resp.choices[0].message.content);
            GenericApiClient::output_parse_pair(output)
        }
    }
}
