//! API adapter for Gemini.
//!
//! Reference: https://ai.google.dev/api

use const_format::concatcp;

use serde::{Deserialize, Serialize};
use serde_json::Number;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;

use crate::utils::error::ApiKeyCheckError;

/// Gemini API request URL prefix.
const GEMINI_API_PREFIX: &str = "https://generativelanguage.googleapis.com/v1beta/";

/// Gemini default model name.
const GEMINI_MODEL_NAME: &str = "gemini-2.0-flash";

/// API key validity check request URL.
/// Accompolished with the model information URL.
const CHECK_API_KEY_URL: &str = concatcp!(GEMINI_API_PREFIX, "models/", GEMINI_MODEL_NAME);

/// Default Gemini API key with no credits (only free quota access).
const FREE_QUOTA_API_KEY: &str = "AIzaSyBz4AFXbOdj_pQ0ai0z_IithH76r9b0sro";

/// Gemini API client.
pub(crate) struct ApiClient {
    api_key: String,
    client: Client,
}

/// Gemini API validation response body.
#[derive(Serialize, Deserialize)]
struct ApiKeyCheckResponse {
    name: String,
    version: String,
}

impl ApiClient {
    /// Creates a new Gemini API client. Only successful if passes the API key validity check.
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
        log::debug!("Choosing the Gemini API...");

        let response = self
            .client
            .get(CHECK_API_KEY_URL)
            .query(&[("key", &self.api_key)])
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
            if !resp.name.ends_with(GEMINI_MODEL_NAME) {
                return Err(ApiKeyCheckError::status(format!(
                    "API key validation successful, but unexpected model name: {}",
                    resp.name
                )));
            }
        }

        Ok(())
    }
}
