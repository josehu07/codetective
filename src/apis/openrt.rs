//! API adapter for OpenRouter.
//!
//! Reference: https://openrouter.ai/docs/api-reference/overview

use const_format::concatcp;

use serde::{Deserialize, Serialize};
use serde_json::Number;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;

use crate::utils::error::{ApiKeyCheckError, ApiMakeCallError};

/// OpenRouter API request URL prefix.
const OPENRT_API_PREFIX: &str = "https://openrouter.ai/api/v1";

/// API key validity check request URL.
/// Accompolished with the rate/credit limit checking API.
const CHECK_API_KEY_URL: &str = concatcp!(OPENRT_API_PREFIX, "/auth/key");

/// Default OpenRouter API key with no credits (only free quota access).
const FREE_QUOTA_API_KEY: &str =
    "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";

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
        unimplemented!()
    }
}
