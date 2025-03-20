//! API adapter for Claude.
//!
//! Reference: https://docs.anthropic.com/en/api/getting-started

use const_format::concatcp;

use serde::{Deserialize, Serialize};
use serde_json::Number;

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client;

use crate::utils::error::ApiKeyCheckError;

/// Claude API request URL prefix.
const CLAUDE_API_PREFIX: &str = "https://api.anthropic.com/v1/";

/// Claude default model name.
const CLAUDE_MODEL_NAME: &str = "claude-3-7-sonnet-20250219";

/// Claude requires an API version date.
const CLAUDE_API_VERSION: &str = "2023-06-01";

/// API key validity check request URL.
/// Accompolished with the model information URL.
const CHECK_API_KEY_URL: &str = concatcp!(CLAUDE_API_PREFIX, "models/", CLAUDE_MODEL_NAME);

/// Claude API client.
pub(crate) struct ApiClient {
    api_key: String,
    client: Client,
}

/// Claude API validation response body.
#[derive(Serialize, Deserialize)]
struct ApiKeyCheckResponse {
    #[serde(rename = "type")]
    o_type: String,
    id: String,
}

impl ApiClient {
    /// Creates a new Claude API client. Only successful if passes the API key validity check.
    pub(crate) async fn new(api_key: Option<String>) -> Result<Self, ApiKeyCheckError> {
        let client = if let Some(api_key) = api_key {
            Self {
                api_key,
                client: Client::new(),
            }
        } else {
            return Err(ApiKeyCheckError::limit(
                "API provider Claude has no free quota available",
            ));
        };

        client.check_api_key().await?;
        Ok(client)
    }

    /// Makes an API key validity check request and returns an error if unsuccessful.
    async fn check_api_key(&self) -> Result<(), ApiKeyCheckError> {
        log::debug!("Choosing the Claude API...");

        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&self.api_key)?);
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(CLAUDE_API_VERSION),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "anthropic-dangerous-direct-browser-access",
            HeaderValue::from_static("true"),
        );

        let response = self
            .client
            .get(CHECK_API_KEY_URL)
            .headers(headers)
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
            if resp.id != CLAUDE_MODEL_NAME {
                return Err(ApiKeyCheckError::status(format!(
                    "API key validation successful, but unexpected model name: {}",
                    resp.id
                )));
            }
        }

        Ok(())
    }
}
