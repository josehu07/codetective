//! Step 1 section: API provider selection and API key.

use leptos::prelude::*;
use leptos::task::spawn_local;

use gloo_timers::future::TimeoutFuture;

use crate::apis::ApiClient;
use crate::utils::error::{ApiKeyCheckError, CodeImportError};
use crate::utils::gadgets::{
    FailureIndicator, HoverInfoIcon, InvisibleIndicator, SpinningIndicator, StepHeaderCollapsed,
    StepHeaderExpanded, SuccessIndicator,
};
use crate::utils::{NBHY, NBSP};
use crate::{CodeGroup, FileResults, StepStage, TaskQueue, ValidationState};

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
            ApiProvider::OpenRt => "OpenRouter (Mistral Large)",
            ApiProvider::GroqCl => "Groq Cloud (Llama-3-70B)",
            ApiProvider::Free => "Free Quota (Preset)",
            ApiProvider::Null => "Null",
        }
    }
}

// Helper functions and handler "closure"s:
fn button_style_classes(is_selected: bool) -> String {
    format!(
        "w-32 h-16 rounded-lg shadow-md hover:shadow-lg flex-col items-center justify-center font-semibold border {}",
        if is_selected { "bg-gray-200 text-gray-900 border-gray-400" } else { "bg-white hover:bg-gray-200 text-gray-600 hover:text-gray-900 border-gray-300" },
    )
}

fn handle_api_select_button(
    api_provider: RwSignal<ApiProvider>,
    api_key_vstate: RwSignal<ValidationState<ApiKeyCheckError>>,
    selected_provider: ApiProvider,
) {
    api_provider.set(selected_provider);
    api_key_vstate.set(ValidationState::Idle);
}

fn handle_api_key_submit(
    api_provider: RwSignal<ApiProvider>,
    input_api_key: RwSignal<String>,
    api_key_vstate: RwSignal<ValidationState<ApiKeyCheckError>>,
    api_client: RwSignal<Option<ApiClient>>,
    stage: RwSignal<StepStage>,
) {
    let current_api_provider = api_provider.get();
    let mut api_key = input_api_key.read().trim().to_string();

    if current_api_provider != ApiProvider::Free && (api_key.is_empty() || !api_key.is_ascii()) {
        log::warn!("API key input field is empty or non-ASCII, please try again...");
        api_key_vstate.set(ValidationState::Failure(ApiKeyCheckError::ascii(
            "API key input is empty or non-ASCII",
        )));
        return;
    } else if current_api_provider == ApiProvider::Free {
        api_key = "preset".to_string();
    }

    api_key_vstate.set(ValidationState::Pending);

    spawn_local(async move {
        log::info!(
            "Step 1 validating: using {} key '{}'...",
            current_api_provider.name(),
            api_key
        );

        match ApiClient::new(current_api_provider, api_key.clone()).await {
            Ok(client) => {
                let chosen_provider = client.provider();
                api_client.set(Some(client));
                api_key_vstate.set(ValidationState::Success);

                // small delay before proceeding to next stage
                TimeoutFuture::new(500).await;

                log::info!(
                    "Step 1 confirmed: using {} key '{}'",
                    chosen_provider.name(),
                    api_key
                );
                stage.set(StepStage::ApiDone);
            }

            Err(err) => {
                log::error!(
                    "API client creation failed for {}: {}",
                    current_api_provider.name(),
                    err
                );
                api_key_vstate.set(ValidationState::Failure(err));
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn handle_back_button(
    api_key_vstate: RwSignal<ValidationState<ApiKeyCheckError>>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
    file_results: RwSignal<FileResults>,
    stage: RwSignal<StepStage>,
) {
    api_key_vstate.set(ValidationState::Idle);
    code_in_vstate.set(ValidationState::Idle);
    code_group.update(|cg| {
        cg.reset();
    });
    task_queue.update(|queue| {
        queue.clear();
    });
    num_finished.set(0);
    detection_cp.set(false);
    file_results.update(|results| {
        results.clear();
    });
    stage.set(StepStage::Initial);

    log::info!("Step 1 rolled back: resetting API key validation stage");
}

// Different display componenets shown selectively:
#[component]
fn ValidationIndicator(
    api_key_vstate: RwSignal<ValidationState<ApiKeyCheckError>>,
) -> impl IntoView {
    move || match api_key_vstate.get() {
        ValidationState::Idle => InvisibleIndicator().into_any(),
        ValidationState::Pending => SpinningIndicator().into_any(),
        ValidationState::Success => SuccessIndicator().into_any(),
        ValidationState::Failure(_) => FailureIndicator().into_any(),
    }
}

#[component]
fn ValidationErrorMsg(
    api_key_vstate: RwSignal<ValidationState<ApiKeyCheckError>>,
) -> impl IntoView {
    move || {
        if let ValidationState::Failure(err) = api_key_vstate.get() {
            Some(view! {
                <div class="text-red-700 text-base font-mono mt-4 text-center animate-fade-in">
                    {format!(
                        "API key validation failed: {}",
                        match &err {
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
    api_provider: RwSignal<ApiProvider>,
    input_api_key: RwSignal<String>,
    api_key_vstate: RwSignal<ValidationState<ApiKeyCheckError>>,
    api_client: RwSignal<Option<ApiClient>>,
    stage: RwSignal<StepStage>,
    placeholder: &'static str,
) -> impl IntoView {
    view! {
        <div class="pt-6 pb-2 px-2 animate-slide-down origin-top">
            <div class="flex items-center justify-center space-x-4">
                <label for="api-key" class="text-base text-gray-900 whitespace-nowrap">
                    Enter API Key:
                </label>
                <input
                    type="password"
                    id="api-key"
                    placeholder=placeholder
                    prop:value=move || input_api_key.get()
                    prop:disabled=move || api_key_vstate.get() == ValidationState::Pending
                    on:input=move |ev| {
                        input_api_key.set(event_target_value(&ev));
                    }
                    on:keydown=move |ev| {
                        if ev.key_code() != 0 && ev.key() == "Enter"
                            && api_key_vstate.get() != ValidationState::Pending
                            && api_key_vstate.get() != ValidationState::Success
                        {
                            handle_api_key_submit(
                                api_provider,
                                input_api_key,
                                api_key_vstate,
                                api_client,
                                stage,
                            );
                        }
                    }
                    class="flex-1 p-2 max-w-xl border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                />

                <HoverInfoIcon text="Codetective is a fully client-side WASM app. Your API key is not exposed to any middle server. Charges apply to your API key, of course." />

                <button
                    on:click=move |_| {
                        if api_key_vstate.get() != ValidationState::Pending
                            && api_key_vstate.get() != ValidationState::Success
                        {
                            handle_api_key_submit(
                                api_provider,
                                input_api_key,
                                api_key_vstate,
                                api_client,
                                stage,
                            );
                        }
                    }
                    disabled=move || api_key_vstate.get() == ValidationState::Pending
                    class=move || {
                        let base = "px-4 py-2 bg-gray-500 text-white rounded-md shadow transition-colors";
                        match api_key_vstate.get() {
                            ValidationState::Pending => {
                                format!("{} opacity-75 cursor-not-allowed", base)
                            }
                            _ => format!("{} hover:bg-gray-600", base),
                        }
                    }
                >
                    Confirm
                </button>

                <ValidationIndicator api_key_vstate />
            </div>

            <ValidationErrorMsg api_key_vstate />
        </div>
    }
}

#[component]
fn FreeApiChoiceSection(
    api_provider: RwSignal<ApiProvider>,
    input_api_key: RwSignal<String>,
    api_key_vstate: RwSignal<ValidationState<ApiKeyCheckError>>,
    api_client: RwSignal<Option<ApiClient>>,
    stage: RwSignal<StepStage>,
) -> impl IntoView {
    view! {
        <div class="pt-6 pb-2 px-2 animate-slide-down origin-top">
            <div class="flex items-center justify-center space-x-4">
                <div class="text-base text-gray-900 whitespace-nowrap">
                    Use a provider of our choice that currently grants limited free{NBHY}tier quota.
                </div>

                <HoverInfoIcon text="Limited availability per minute, day, and/or month, of course." />

                <button
                    on:click=move |_| {
                        if api_key_vstate.get() != ValidationState::Pending
                            && api_key_vstate.get() != ValidationState::Success
                        {
                            handle_api_key_submit(
                                api_provider,
                                input_api_key,
                                api_key_vstate,
                                api_client,
                                stage,
                            );
                        }
                    }
                    disabled=move || api_key_vstate.get() == ValidationState::Pending
                    class=move || {
                        let base = "px-5 py-2 bg-gray-500 text-white rounded-md shadow transition-colors";
                        match api_key_vstate.get() {
                            ValidationState::Pending => {
                                format!("{} opacity-75 cursor-not-allowed", base)
                            }
                            _ => format!("{} hover:bg-gray-600", base),
                        }
                    }
                >
                    Confirm
                </button>

                <ValidationIndicator api_key_vstate />
            </div>

            <ValidationErrorMsg api_key_vstate />
        </div>
    }
}

#[component]
fn ApiSelectionExpandedView(
    api_provider: RwSignal<ApiProvider>,
    input_api_key: RwSignal<String>,
    api_key_vstate: RwSignal<ValidationState<ApiKeyCheckError>>,
    api_client: RwSignal<Option<ApiClient>>,
    stage: RwSignal<StepStage>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm animate-fade-in">
            <StepHeaderExpanded step=1 />

            <div class="text-xl text-center text-gray-900">Select API Provider...</div>

            <div class="flex space-x-6 mt-6 mb-2 justify-center">
                <button
                    on:click=move |_| handle_api_select_button(
                        api_provider,
                        api_key_vstate,
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
                        api_provider,
                        api_key_vstate,
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
                        api_provider,
                        api_key_vstate,
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
                        api_provider,
                        api_key_vstate,
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
                        api_provider,
                        api_key_vstate,
                        ApiProvider::OpenRt,
                    )
                    class=move || button_style_classes(api_provider.get() == ApiProvider::OpenRt)
                >
                    OpenRouter
                    <br />
                    <div class="font-mono">mistral</div>
                </button>
            </div>

            {move || {
                (api_provider.get() == ApiProvider::Free)
                    .then_some(
                        view! {
                            <FreeApiChoiceSection
                                api_provider
                                input_api_key
                                api_key_vstate
                                api_client
                                stage
                            />
                        },
                    )
            }}

            {move || {
                (api_provider.get() == ApiProvider::OpenAI)
                    .then_some(
                        view! {
                            <ApiKeyInputSection
                                api_provider
                                input_api_key
                                api_key_vstate
                                api_client
                                stage
                                placeholder="sk-..."
                            />
                        },
                    )
            }}

            {move || {
                (api_provider.get() == ApiProvider::Claude)
                    .then_some(
                        view! {
                            <ApiKeyInputSection
                                api_provider
                                input_api_key
                                api_key_vstate
                                api_client
                                stage
                                placeholder="sk-..."
                            />
                        },
                    )
            }}

            {move || {
                (api_provider.get() == ApiProvider::Gemini)
                    .then_some(
                        view! {
                            <ApiKeyInputSection
                                api_provider
                                input_api_key
                                api_key_vstate
                                api_client
                                stage
                                placeholder="AI..."
                            />
                        },
                    )
            }}

            {move || {
                (api_provider.get() == ApiProvider::OpenRt)
                    .then_some(
                        view! {
                            <ApiKeyInputSection
                                api_provider
                                input_api_key
                                api_key_vstate
                                api_client
                                stage
                                placeholder="sk-or-..."
                            />
                        },
                    )
            }}
        </div>
    }
}

#[component]
fn ApiSelectionCollapsedView(
    api_provider: RwSignal<ApiProvider>,
    api_key_vstate: RwSignal<ValidationState<ApiKeyCheckError>>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
    file_results: RwSignal<FileResults>,
    stage: RwSignal<StepStage>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm">
            <StepHeaderCollapsed step=1 />

            <div class="text-center text-gray-800 text-lg">
                <span class="font-semibold">API Provider and Model:{NBSP}{NBSP}</span>
                <span class="text-xl font-mono">{move || api_provider.get().name()}</span>
            </div>

            {move || {
                (api_provider.get() != ApiProvider::Null)
                    .then_some(
                        view! {
                            <button
                                on:click=move |_| handle_back_button(
                                    api_key_vstate,
                                    code_in_vstate,
                                    code_group,
                                    task_queue,
                                    num_finished,
                                    detection_cp,
                                    file_results,
                                    stage,
                                )
                                class="absolute -bottom-3 -right-5 px-4 py-2 bg-gray-500 hover:bg-gray-600 rounded-md flex items-center justify-center align-middle text-white transition-colors"
                            >
                                Back
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    class="w-5 h-5 ml-1"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                    stroke="currentColor"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M14 17C16.7614 17 19 14.7614 19 12C19 9.23858 16.7614 7 14 7H8M8 7L11 4M8 7L11 10"
                                    />
                                </svg>
                            </button>
                        },
                    )
            }}
        </div>
    }
}

/// The API selection step wrapped in one place.
#[component]
pub(crate) fn ApiSelection(
    api_provider: RwSignal<ApiProvider>,
    input_api_key: RwSignal<String>,
    api_key_vstate: RwSignal<ValidationState<ApiKeyCheckError>>,
    api_client: RwSignal<Option<ApiClient>>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
    file_results: RwSignal<FileResults>,
    stage: RwSignal<StepStage>,
) -> impl IntoView {
    view! {
        {move || {
            (stage.get() == StepStage::Initial)
                .then_some(
                    view! {
                        <ApiSelectionExpandedView
                            api_provider
                            input_api_key
                            api_key_vstate
                            api_client
                            stage
                        />
                    },
                )
        }}

        {move || {
            (stage.get() > StepStage::Initial)
                .then_some(
                    view! {
                        <ApiSelectionCollapsedView
                            api_provider
                            api_key_vstate
                            code_in_vstate
                            code_group
                            task_queue
                            num_finished
                            detection_cp
                            file_results
                            stage
                        />
                    },
                )
        }}
    }
}
