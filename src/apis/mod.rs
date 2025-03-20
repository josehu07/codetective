//! API adapters for various AI model providers.

use crate::api_selection::ApiProvider;
use crate::utils::error::ApiKeyCheckError;

mod claude;
mod gemini;
mod groqcl;
mod openai;
mod openrt;

/// "Generic" API client.
pub(crate) enum ApiClient {
    OpenAI(openai::ApiClient),
    Claude(claude::ApiClient),
    Gemini(gemini::ApiClient),
    OpenRt(openrt::ApiClient),
    GroqCl(groqcl::ApiClient),
}

impl ApiClient {
    pub(crate) fn provider(&self) -> ApiProvider {
        match self {
            Self::OpenAI(_) => ApiProvider::OpenAI,
            Self::Claude(_) => ApiProvider::Claude,
            Self::Gemini(_) => ApiProvider::Gemini,
            Self::OpenRt(_) => ApiProvider::OpenRt,
            Self::GroqCl(_) => ApiProvider::GroqCl,
        }
    }
}

impl ApiClient {
    /// Creates a new API client for the given provider with the given API key.
    /// Only successful if passes the API key validity check.
    pub(crate) async fn new(
        provider: ApiProvider,
        api_key: String,
    ) -> Result<Self, ApiKeyCheckError> {
        // some adapters support a free-quota API key when key not given
        assert_ne!(provider, ApiProvider::Null);
        let api_key = (provider != ApiProvider::Free).then_some(api_key);

        match provider {
            ApiProvider::OpenAI => Ok(Self::OpenAI(openai::ApiClient::new(api_key).await?)),
            ApiProvider::Claude => Ok(Self::Claude(claude::ApiClient::new(api_key).await?)),
            ApiProvider::Gemini => Ok(Self::Gemini(gemini::ApiClient::new(api_key).await?)),
            ApiProvider::OpenRt => Ok(Self::OpenRt(openrt::ApiClient::new(api_key).await?)),
            ApiProvider::GroqCl => Ok(Self::GroqCl(groqcl::ApiClient::new(api_key).await?)),

            ApiProvider::Free => {
                // randomly choose an adapter that might have free quota availability
                let freeable_providers = [
                    ApiProvider::Gemini,
                    ApiProvider::OpenRt,
                    ApiProvider::GroqCl,
                ];
                let provider_idx = (getrandom::u32()? as usize) % freeable_providers.len();

                match freeable_providers[provider_idx] {
                    ApiProvider::Gemini => Ok(Self::Gemini(gemini::ApiClient::new(api_key).await?)),
                    ApiProvider::OpenRt => Ok(Self::OpenRt(openrt::ApiClient::new(api_key).await?)),
                    ApiProvider::GroqCl => Ok(Self::GroqCl(groqcl::ApiClient::new(api_key).await?)),
                    _ => unreachable!(),
                }
            }

            _ => unreachable!(),
        }
    }
}
