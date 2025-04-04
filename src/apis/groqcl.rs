//! API adapter for Groq Cloud.
//!
//! Reference: https://console.groq.com/docs/api-reference

use std::mem;

use const_format::concatcp;

use serde::{Deserialize, Serialize};

use reqwest::header::CONTENT_TYPE;
use reqwest::Client;

use crate::apis::ApiClient as GenericApiClient;
use crate::utils::error::{ApiKeyCheckError, ApiMakeCallError};

/// Groq Cloud API request URL prefix.
const GROQCL_API_PREFIX: &str = "https://api.groq.com/openai/v1";

/// Groq Cloud default model name.
const GROQCL_MODEL_NAME: &str = "llama3-70b-8192";

/// API key validity check request URL.
/// Accompolished with the model information URL.
const CHECK_API_KEY_URL: &str = concatcp!(GROQCL_API_PREFIX, "/models/", GROQCL_MODEL_NAME);

/// API chat completion request URL.
const CHAT_COMPLETION_URL: &str = concatcp!(GROQCL_API_PREFIX, "/chat/completions");

/// Default Groq Cloud API key with no credits (only free quota access).
const FREE_QUOTA_API_KEY: &str = "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";

/// Max output tokens cap.
const MAX_OUTPUT_TOKENS: u32 = 500;

/// Groq Cloud API client.
pub(crate) struct ApiClient {
    api_key: String,
    client: Client,
}

/// Groq Cloud API validation response body.
#[derive(Serialize, Deserialize, Debug)]
struct ApiKeyCheckResponse {
    id: String,
    object: String,
    active: bool,
}

/// Groq Cloud detection API call response body.
#[derive(Serialize, Deserialize, Debug)]
struct ApiDetectionResponse {
    id: String,
    model: String,
    choices: Vec<ApiDetectionResponseChoice>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiDetectionResponseChoice {
    message: ApiDetectionResponseMessage,
    finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiDetectionResponseMessage {
    content: String,
}

impl ApiClient {
    /// Creates a new Groq Cloud API client. Only successful if passes the API key validity check.
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
        log::debug!("Choosing the Groq Cloud API...");

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
            let resp = response.json::<ApiKeyCheckResponse>().await?;
            if resp.id != GROQCL_MODEL_NAME {
                return Err(ApiKeyCheckError::status(format!(
                    "API key validation successful, but unexpected model name: {}",
                    resp.id
                )));
            }
            if !resp.active {
                return Err(ApiKeyCheckError::status(format!(
                    "API key validation successful, but model {} is inactive",
                    GROQCL_MODEL_NAME
                )));
            }
        }

        Ok(())
    }

    /// Makes an detection API call and returns the response.
    pub(crate) async fn call(&self, prompt: String) -> Result<(u8, String), ApiMakeCallError> {
        log::debug!("Making API call to Groq Cloud...");

        let request = serde_json::json!({
            "model": GROQCL_MODEL_NAME,
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "max_completion_tokens": MAX_OUTPUT_TOKENS,
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
