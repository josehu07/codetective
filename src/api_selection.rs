//! Step 1 section: API provider selection and API key.

use leptos::prelude::*;
use leptos::task::spawn_local;

use gloo_timers::future::TimeoutFuture;

use crate::apis::ApiClient;
use crate::utils::error::ApiKeyCheckError;
use crate::utils::gadgets::{
    button_style_classes, FailureIndicator, HoverInfoIcon, InvisibleIndicator, SpinningIndicator,
    StepHeaderCollapsed, StepHeaderExpanded, SuccessIndicator,
};
use crate::utils::{NBHY, NBSP};
use crate::Stage;

/// Enum that controls the state of API provider selection.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum ApiProvider {
    OpenAI,
    Claude,
    Gemini,
    OpenRt,
    GroqCl,
    Free,
    Null,
}

impl ApiProvider {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            ApiProvider::OpenAI => "OpenAI (GPT-4o)",
            ApiProvider::Claude => "Claude (3.7 Sonnet)",
            ApiProvider::Gemini => "Gemini (2.0 Flash)",
            ApiProvider::OpenRt => "OpenRouter (Auto)",
            ApiProvider::GroqCl => "Groq (Llama-3-70B)",
            ApiProvider::Free => "Free Quota (Preset)",
            ApiProvider::Null => "Null",
        }
    }
}

/// API key validation state.
#[derive(Clone, PartialEq, Debug)]
enum ValidationState {
    Idle,
    Pending,
    Success,
    Failure(ApiKeyCheckError),
}

// Helper functions and handler "closure"s:
fn handle_api_select_button(
    set_api_provider: WriteSignal<ApiProvider>,
    set_validation_state: WriteSignal<ValidationState>,
    selected_provider: ApiProvider,
) {
    set_api_provider.set(selected_provider);
    set_validation_state.set(ValidationState::Idle);
}

fn handle_api_key_submit(
    api_provider: ReadSignal<ApiProvider>,
    input_api_key: ReadSignal<String>,
    set_validation_state: WriteSignal<ValidationState>,
    set_api_client: WriteSignal<Option<ApiClient>>,
    set_stage: WriteSignal<Stage>,
) {
    let current_api_provider = api_provider.get();
    let api_key = input_api_key.read().trim().to_string();

    if current_api_provider != ApiProvider::Free && (api_key.is_empty() || !api_key.is_ascii()) {
        log::warn!("API key input field is empty or non-ASCII, please try again...");
        set_validation_state.set(ValidationState::Failure(ApiKeyCheckError::ascii(
            "API key iput is empty or non-ASCII",
        )));
        return;
    }

    set_validation_state.set(ValidationState::Pending);

    spawn_local(async move {
        log::info!(
            "Step 1 validating: using {} key '{}'...",
            current_api_provider.name(),
            api_key
        );

        match ApiClient::new(current_api_provider, api_key.clone()).await {
            Ok(client) => {
                let chosen_provider = client.provider();
                set_api_client.set(Some(client));
                set_validation_state.set(ValidationState::Success);

                // small delay before proceeding to next stage
                TimeoutFuture::new(500).await;

                log::info!(
                    "Step 1 confirmed: using {} key '{}'",
                    chosen_provider.name(),
                    api_key
                );
                set_stage.set(Stage::ApiProvided);
            }

            Err(err) => {
                log::error!(
                    "API client creation failed for {}: {}",
                    current_api_provider.name(),
                    err
                );
                set_validation_state.set(ValidationState::Failure(err));
            }
        }
    });
}

fn handle_confirm_button(
    api_provider: ReadSignal<ApiProvider>,
    input_api_key: ReadSignal<String>,
    validation_state: ReadSignal<ValidationState>,
    set_validation_state: WriteSignal<ValidationState>,
    set_api_client: WriteSignal<Option<ApiClient>>,
    set_stage: WriteSignal<Stage>,
) {
    if validation_state.get() != ValidationState::Pending
        && validation_state.get() != ValidationState::Success
    {
        handle_api_key_submit(
            api_provider,
            input_api_key,
            set_validation_state,
            set_api_client,
            set_stage,
        );
    }
}

fn handle_enter_key_down(
    api_provider: ReadSignal<ApiProvider>,
    input_api_key: ReadSignal<String>,
    validation_state: ReadSignal<ValidationState>,
    set_validation_state: WriteSignal<ValidationState>,
    set_api_client: WriteSignal<Option<ApiClient>>,
    set_stage: WriteSignal<Stage>,
) {
    if validation_state.get() != ValidationState::Pending
        && validation_state.get() != ValidationState::Success
    {
        handle_api_key_submit(
            api_provider,
            input_api_key,
            set_validation_state,
            set_api_client,
            set_stage,
        );
    }
}

fn handle_back_button(
    set_api_provider: WriteSignal<ApiProvider>,
    set_input_api_key: WriteSignal<String>,
    set_validation_state: WriteSignal<ValidationState>,
    set_stage: WriteSignal<Stage>,
) {
    set_api_provider.set(ApiProvider::Null);
    set_input_api_key.set(String::new());
    set_validation_state.set(ValidationState::Idle);
    set_stage.set(Stage::Initial);

    log::info!("Step 1 rolled back: resetting API provider and key");
}

// Different display componenets shown selectively:
#[component]
fn ValidationIndicator(validation_state: ReadSignal<ValidationState>) -> impl IntoView {
    move || match validation_state.get() {
        ValidationState::Idle => InvisibleIndicator().into_any(),
        ValidationState::Pending => SpinningIndicator().into_any(),
        ValidationState::Success => SuccessIndicator().into_any(),
        ValidationState::Failure(_) => FailureIndicator().into_any(),
    }
}

#[component]
fn ValidationErrorMsg(validation_state: ReadSignal<ValidationState>) -> impl IntoView {
    move || {
        if let ValidationState::Failure(err) = validation_state.get() {
            Some(view! {
                <div class="text-red-700 text-base font-mono mt-4 text-center animate-fade-in">
                    {format!(
                        "API key validation failed: {}",
                        match err {
                            ApiKeyCheckError::Parse(_) => "internal parsing error...",
                            ApiKeyCheckError::Status(_) => "authorization failure, invalid API key?",
                            ApiKeyCheckError::Limit(_) => "usage limit seems to have been exceeded!",
                            ApiKeyCheckError::Ascii(_) => "please provide a legit API key...",
                            ApiKeyCheckError::Random(_) => "random number generation error...",
                        },
                    )}
                </div>
            })
        } else {
            None
        }
    }
}

#[component]
fn ApiKeyInputSection(
    api_provider: ReadSignal<ApiProvider>,
    input_api_key: ReadSignal<String>,
    set_input_api_key: WriteSignal<String>,
    validation_state: ReadSignal<ValidationState>,
    set_validation_state: WriteSignal<ValidationState>,
    set_api_client: WriteSignal<Option<ApiClient>>,
    set_stage: WriteSignal<Stage>,
    placeholder: &'static str,
) -> impl IntoView {
    view! {
        <div class="pt-6 pb-2 px-2 overflow-hidden animate-slide-down origin-top">
            <div class="flex items-center justify-center space-x-4">
                <label for="api-key" class="text-base text-gray-900 whitespace-nowrap">
                    Enter API Key:
                </label>
                <input
                    type="password"
                    id="api-key"
                    placeholder=placeholder
                    prop:value=move || input_api_key.get()
                    prop:disabled=move || validation_state.get() == ValidationState::Pending
                    on:input=move |ev| {
                        set_input_api_key.set(event_target_value(&ev));
                    }
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            handle_enter_key_down(
                                api_provider,
                                input_api_key,
                                validation_state,
                                set_validation_state,
                                set_api_client,
                                set_stage,
                            );
                        }
                    }
                    class="flex-1 p-2 max-w-xl border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                />

                <HoverInfoIcon text="Codetective is a fully client-side WASM app. Your API key is not exposed to any middle server. Charges apply to your API key, of course." />

                <button
                    on:click=move |_| {
                        handle_confirm_button(
                            api_provider,
                            input_api_key,
                            validation_state,
                            set_validation_state,
                            set_api_client,
                            set_stage,
                        );
                    }
                    disabled=move || validation_state.get() == ValidationState::Pending
                    class=move || {
                        let base = "px-4 py-2 bg-gray-500 text-white rounded-md shadow transition-colors";
                        match validation_state.get() {
                            ValidationState::Pending => {
                                format!("{} opacity-75 cursor-not-allowed", base)
                            }
                            _ => format!("{} hover:bg-gray-600", base),
                        }
                    }
                >
                    Confirm
                </button>

                <ValidationIndicator validation_state />
            </div>

            <ValidationErrorMsg validation_state />
        </div>
    }
}

#[component]
fn FreeApiChoiceSection(
    api_provider: ReadSignal<ApiProvider>,
    input_api_key: ReadSignal<String>,
    validation_state: ReadSignal<ValidationState>,
    set_validation_state: WriteSignal<ValidationState>,
    set_api_client: WriteSignal<Option<ApiClient>>,
    set_stage: WriteSignal<Stage>,
) -> impl IntoView {
    view! {
        <div class="pt-6 pb-2 px-2 overflow-hidden animate-slide-down origin-top">
            <div class="flex items-center justify-center space-x-4">
                <div class="text-base text-gray-900 whitespace-nowrap">
                    Use a provider of our choice that currently grants limited free{NBHY}
                    tier quota.
                </div>

                <HoverInfoIcon text="Limited availability per minute, day, and/or month, of course." />

                <button
                    on:click=move |_| {
                        handle_confirm_button(
                            api_provider,
                            input_api_key,
                            validation_state,
                            set_validation_state,
                            set_api_client,
                            set_stage
                        );
                    }
                    disabled=move || validation_state.get() == ValidationState::Pending
                    class=move || {
                        let base = "px-5 py-2 bg-gray-500 text-white rounded-md shadow transition-colors";
                        match validation_state.get() {
                            ValidationState::Pending => {
                                format!("{} opacity-75 cursor-not-allowed", base)
                            }
                            _ => format!("{} hover:bg-gray-600", base),
                        }
                    }
                >
                    Confirm
                </button>

                <ValidationIndicator validation_state />
            </div>

            <ValidationErrorMsg validation_state />
        </div>
    }
}

#[component]
fn ApiSelectionExpandedView(
    api_provider: ReadSignal<ApiProvider>,
    set_api_provider: WriteSignal<ApiProvider>,
    input_api_key: ReadSignal<String>,
    set_input_api_key: WriteSignal<String>,
    validation_state: ReadSignal<ValidationState>,
    set_validation_state: WriteSignal<ValidationState>,
    set_api_client: WriteSignal<Option<ApiClient>>,
    set_stage: WriteSignal<Stage>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm animate-fade-in">
            <StepHeaderExpanded step=1 />

            <div class="text-xl text-center text-gray-900">Select API Provider...</div>

            <div class="flex space-x-6 mt-6 mb-2 justify-center">
                <button
                    on:click=move |_| handle_api_select_button(
                        set_api_provider,
                        set_validation_state,
                        ApiProvider::Free,
                    )
                    class=move || button_style_classes(api_provider.get() == ApiProvider::Free)
                >
                    Free API
                    <br />
                    <div class="font-mono">limited</div>
                </button>

                <button
                    on:click=move |_| handle_api_select_button(
                        set_api_provider,
                        set_validation_state,
                        ApiProvider::OpenAI,
                    )
                    class=move || button_style_classes(api_provider.get() == ApiProvider::OpenAI)
                >
                    OpenAI
                    <br />
                    <div class="font-mono">gpt{NBHY}4o</div>
                </button>

                <button
                    on:click=move |_| handle_api_select_button(
                        set_api_provider,
                        set_validation_state,
                        ApiProvider::Claude,
                    )
                    class=move || button_style_classes(api_provider.get() == ApiProvider::Claude)
                >
                    Claude
                    <br />
                    <div class="font-mono">3.7{NBHY}sonnet</div>
                </button>

                <button
                    on:click=move |_| handle_api_select_button(
                        set_api_provider,
                        set_validation_state,
                        ApiProvider::Gemini,
                    )
                    class=move || button_style_classes(api_provider.get() == ApiProvider::Gemini)
                >
                    Gemini
                    <br />
                    <div class="font-mono">2.0{NBHY}flash</div>
                </button>

                <button
                    on:click=move |_| handle_api_select_button(
                        set_api_provider,
                        set_validation_state,
                        ApiProvider::OpenRt,
                    )
                    class=move || button_style_classes(api_provider.get() == ApiProvider::OpenRt)
                >
                    OpenRouter
                    <br />
                    <div class="font-mono">auto</div>
                </button>
            </div>

            {move || (api_provider.get() == ApiProvider::Free).then_some(view! {
                <FreeApiChoiceSection
                    api_provider
                    input_api_key
                    validation_state
                    set_validation_state
                    set_api_client
                    set_stage
                />
            })}

            {move || (api_provider.get() == ApiProvider::OpenAI).then_some(view! {
                <ApiKeyInputSection
                    api_provider
                    input_api_key
                    set_input_api_key
                    validation_state
                    set_validation_state
                    set_api_client
                    set_stage
                    placeholder="sk-..."
                />
            })}

            {move || (api_provider.get() == ApiProvider::Claude).then_some(view! {
                <ApiKeyInputSection
                    api_provider
                    input_api_key
                    set_input_api_key
                    validation_state
                    set_validation_state
                    set_api_client
                    set_stage
                    placeholder="sk-..."
                />
            })}

            {move || (api_provider.get() == ApiProvider::Gemini).then_some(view! {
                <ApiKeyInputSection
                    api_provider
                    input_api_key
                    set_input_api_key
                    validation_state
                    set_validation_state
                    set_api_client
                    set_stage
                    placeholder="AI..."
                />
            })}

            {move || (api_provider.get() == ApiProvider::OpenRt).then_some(view! {
                <ApiKeyInputSection
                    api_provider
                    input_api_key
                    set_input_api_key
                    validation_state
                    set_validation_state
                    set_api_client
                    set_stage
                    placeholder="sk-or-..."
                />
            })}
        </div>
    }
}

#[component]
fn ApiSelectionCollapsedView(
    api_provider: ReadSignal<ApiProvider>,
    set_api_provider: WriteSignal<ApiProvider>,
    set_input_api_key: WriteSignal<String>,
    set_validation_state: WriteSignal<ValidationState>,
    set_stage: WriteSignal<Stage>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm">
            <StepHeaderCollapsed step=1 />

            <div class="text-center text-gray-800 text-lg">
                <span class="font-semibold">API Provider and Model:{NBSP}{NBSP}</span>
                <span class="text-xl font-mono">{move || api_provider.get().name()}</span>
            </div>

            {move || (api_provider.get() != ApiProvider::Null).then_some(view! {
                <button
                    on:click=move |_| handle_back_button(
                        set_api_provider,
                        set_input_api_key,
                        set_validation_state,
                        set_stage
                    )
                    class="absolute -bottom-3 -right-5 px-4 py-2 bg-gray-500 hover:bg-gray-600 rounded-md flex items-center justify-center text-white transition-colors"
                >
                    Back
                </button>
            })}
        </div>
    }
}

/// The API selection step wrapped in one place.
#[component]
pub(crate) fn ApiSelection(
    set_api_client: WriteSignal<Option<ApiClient>>,
    stage: ReadSignal<Stage>,
    set_stage: WriteSignal<Stage>,
) -> impl IntoView {
    let (api_provider, set_api_provider) = signal(ApiProvider::Null);
    let (input_api_key, set_input_api_key) = signal(String::new());
    let (validation_state, set_validation_state) = signal(ValidationState::Idle);

    view! {
        {move || (stage.get() == Stage::Initial).then_some(view! {
            <ApiSelectionExpandedView
                api_provider
                set_api_provider
                input_api_key
                set_input_api_key
                validation_state
                set_validation_state
                set_api_client
                set_stage
            />
        })}

        {move || (stage.get() > Stage::Initial).then_some(view! {
            <ApiSelectionCollapsedView
                api_provider
                set_api_provider
                set_input_api_key
                set_validation_state
                set_stage
            />
        })}
    }
}
