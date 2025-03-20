//! API adapters for various AI model providers.

use std::error::Error;

use crate::api_selection::ApiProvider;
use crate::utils::error::ApiKeyCheckError;

mod openrt;

/// "Generic" API client.
pub(crate) enum ApiClient {
    OpenRt(openrt::ApiClient),
}

impl ApiClient {
    /// Creates a new API client for the given provider with the given API key.
    /// Only successful if passes the API key validity check.
    pub(crate) async fn new(
        provider: ApiProvider,
        api_key: String,
    ) -> Result<Self, ApiKeyCheckError> {
        match provider {
            ApiProvider::Free => Ok(Self::OpenRt(openrt::ApiClient::new().await?)),
            _ => unimplemented!(),
        }
    }
}
