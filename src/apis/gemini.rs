//! API adapter for Gemini.
//!
//! Reference: https://ai.google.dev/api

use std::cmp;
use std::mem;

use const_format::concatcp;

use serde::{Deserialize, Serialize};

use reqwest::header::CONTENT_TYPE;
use reqwest::Client;

use crate::apis::DetectionResultPair;
use crate::utils::error::{ApiKeyCheckError, ApiMakeCallError};

/// Gemini API request URL prefix.
const GEMINI_API_PREFIX: &str = "https://generativelanguage.googleapis.com/v1";

/// Gemini default model name.
const GEMINI_MODEL_NAME: &str = "gemini-2.0-flash";

/// API key validity check request URL.
/// Accompolished with the model information URL.
const CHECK_API_KEY_URL: &str = concatcp!(GEMINI_API_PREFIX, "/models/", GEMINI_MODEL_NAME);

/// API chat completion request URL.
const CHAT_COMPLETION_URL: &str = concatcp!(
    GEMINI_API_PREFIX,
    "/models/",
    GEMINI_MODEL_NAME,
    ":generateContent"
);

/// Default Gemini API key with no credits (only free quota access).
const FREE_QUOTA_API_KEY: &str = "AIzaSyBz4AFXbOdj_pQ0ai0z_IithH76r9b0sro";

/// Max output tokens cap.
const MAX_OUTPUT_TOKENS: u32 = 500;

/// Gemini API client.
pub(crate) struct ApiClient {
    api_key: String,
    client: Client,
}

/// Gemini API validation response body.
#[derive(Serialize, Deserialize, Debug)]
struct ApiKeyCheckResponse {
    name: String,
    version: String,
}

/// Gemini detection API call response body.
#[derive(Serialize, Deserialize, Debug)]
struct ApiDetectionResponse {
    candidates: Vec<ApiDetectionResponseCandidate>,
    #[serde(rename = "modelVersion")]
    model_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiDetectionResponseCandidate {
    content: ApiDetectionResponseContent,
    #[serde(rename = "finishReason")]
    finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiDetectionResponseContent {
    parts: Vec<ApiDetectionResponseContentPart>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiDetectionResponseContentPart {
    text: String,
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

    /// Makes an detection API call and returns the response.
    pub(crate) async fn call(&self, prompt: String) -> Result<(u8, String), ApiMakeCallError> {
        log::debug!("Making API call to Gemini...");

        let request = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
                "maxOutputTokens": MAX_OUTPUT_TOKENS,
            }
        });

        let response = self
            .client
            .post(CHAT_COMPLETION_URL)
            .query(&[("key", &self.api_key)])
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
            if resp.candidates.is_empty() {
                return Err(ApiMakeCallError::parse("no candidates found in response"));
            }
            if resp.candidates[0].content.parts.is_empty() {
                return Err(ApiMakeCallError::parse(
                    "no content parts found in response",
                ));
            }

            // concatinate the text output of all parts (this is likely unnecessary)
            let mut output = mem::take(&mut resp.candidates[0].content.parts[0].text);
            for part in resp.candidates[0].content.parts.iter_mut().skip(1) {
                output.push(' ');
                output.push_str(&part.text);
            }
            self.output_parse_pair(output)
        }
    }

    /// Strip out and parse the expected json output piece from the response.
    fn output_parse_pair(&self, output: String) -> Result<(u8, String), ApiMakeCallError> {
        if let Some(pos_s) = output.find('{') {
            if let Some(pos_e) = output[pos_s..].find('}') {
                let pos_e = pos_s + pos_e;
                let json_str = &output[pos_s..=pos_e];

                let result = serde_json::from_str::<DetectionResultPair>(json_str)?;
                let score = match result.score.as_u64() {
                    Some(n) => cmp::min(n, 100) as u8,
                    _ => return Err(ApiMakeCallError::parse("invalid percentage score value")),
                };
                return Ok((score, result.reason));
            }
        }

        Err(ApiMakeCallError::parse(
            "failed to parse expected json pair from response",
        ))
    }
}
