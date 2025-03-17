//! Step 1 section: API provider selection and API key.

use leptos::prelude::*;
use leptos::web_sys::KeyboardEvent;

use super::{Stage, NBHY, NBSP};

/// Enum that controls the state of API provider selection.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum ApiProvider {
    OpenAI,
    Claude,
    Gemini,
    Free,
    Null,
}

impl ApiProvider {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            ApiProvider::OpenAI => "OpenAI (GPT-4o)",
            ApiProvider::Claude => "Claude (3.7 Sonnet)",
            ApiProvider::Gemini => "Gemini (2.0 Flash)",
            ApiProvider::Free => "Free Quota",
            ApiProvider::Null => "Null",
        }
    }
}

#[component]
pub(crate) fn ApiSelection(
    api_provider: ReadSignal<ApiProvider>,
    set_api_provider: WriteSignal<ApiProvider>,
    set_api_key: WriteSignal<String>,
    stage: ReadSignal<Stage>,
    set_stage: WriteSignal<Stage>,
) -> impl IntoView {
    // for API provider buttons
    let handle_api_button_openai = move |_| {
        set_api_provider.set(ApiProvider::OpenAI);
    };
    let handle_api_button_claude = move |_| {
        set_api_provider.set(ApiProvider::Claude);
    };
    let handle_api_button_gemini = move |_| {
        set_api_provider.set(ApiProvider::Gemini);
    };
    let handle_api_button_free = move |_| {
        set_api_provider.set(ApiProvider::Free);
    };

    let button_style_classes = move |selected_provider: ApiProvider| -> String {
        let is_selected = api_provider.get() == selected_provider;
        format!(
            "w-32 h-16 rounded-lg shadow-md hover:shadow-lg flex-col items-center justify-center font-semibold border {}",
            if is_selected { "bg-gray-200 text-gray-900 border-gray-400" } else { "bg-white hover:bg-gray-200 text-gray-600 hover:text-gray-900 border-gray-300" },
        )
    };

    // for API key text box and submit button
    let (input_api_key, set_input_api_key) = signal(String::new());
    let handle_api_key_submit = move || {
        set_api_key.set(input_api_key.get());
        set_stage.set(Stage::ApiProvided);
        log::info!(
            "Step 1 confirmed: using {} key '{}'",
            api_provider.get().name(),
            input_api_key.get()
        );
    };

    let handle_confirm_button = move |_| {
        handle_api_key_submit();
    };
    let handle_enter_key_down = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            handle_api_key_submit();
        }
    };

    // shown when needing an API key input
    let api_key_input_sec = move |placeholder: &'static str| {
        view! {
            <div class="pt-6 pb-2 px-2 overflow-hidden animate-slide-down origin-top">
                <div class="flex items-center justify-center space-x-4">
                    <label for="api-key" class="text-base text-gray-900 whitespace-nowrap">Enter API Key:</label>
                    <input
                        type="password"
                        id="api-key"
                        placeholder={placeholder}
                        prop:value=move || input_api_key.get()
                        on:input=move |ev| {
                            set_input_api_key.set(event_target_value(&ev));
                        }
                        on:keydown=handle_enter_key_down
                        class="flex-1 p-2 max-w-xl border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <button
                        on:click=handle_confirm_button
                        class="px-4 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-md shadow transition-colors">
                        Confirm
                    </button>
                    <div class="h-5 w-5 text-gray-500 hover:text-gray-700 cursor-help" title="Codetective is a fully client-side WASM application. Your API key is not being exposed to any middle server.">
                        <svg xmlns="http://www.w3.org/2000/svg"
                             fill="none"
                             viewBox="0 0 24 24"
                             stroke="currentColor">
                            <path stroke-linecap="round"
                                  stroke-linejoin="round"
                                  stroke-width="2"
                                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                    </div>
                </div>
            </div>
        }
    };

    // shown when selecting the free quota API
    let api_key_free_choice = move || {
        view! {
            <div class="pt-6 pb-2 px-2 overflow-hidden animate-slide-down origin-top">
                <div class="flex items-center justify-center space-x-4">
                    <div class="text-base text-gray-900 whitespace-nowrap">
                        Use a provider of our choice that currently grants limited free{NBHY}tier quota.
                    </div>
                    <button
                        on:click=handle_confirm_button
                        class="px-5 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-md shadow transition-colors">
                        Confirm
                    </button>
                    <div class="h-5 w-5 text-gray-500 hover:text-gray-700 cursor-help" title="Limited availability per minute, day, and/or month, of course.">
                        <svg xmlns="http://www.w3.org/2000/svg"
                             fill="none"
                             viewBox="0 0 24 24"
                             stroke="currentColor">
                            <path stroke-linecap="round"
                                  stroke-linejoin="round"
                                  stroke-width="2"
                                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                    </div>
                </div>
            </div>
        }
    };

    // for the back button functionality
    let handle_back_button = move |_| {
        set_api_provider.set(ApiProvider::Null);
        set_api_key.set(String::new());
        set_stage.set(Stage::Initial);
        log::info!("Step 1 rolled back: resetting API provider and key");
    };

    // expanded view when this step is active
    let expanded_view = move || {
        view! {
            <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm animate-fade-in">
                <div class="absolute -top-3 -left-5 px-4 py-2 bg-gray-600 rounded-full flex items-center justify-center text-xl text-white font-semibold">
                    Step 1
                </div>

                <div class="text-xl text-center text-gray-900">Select API Provider...</div>

                <div class="flex space-x-6 mt-6 mb-2 justify-center">
                    <button
                        on:click=handle_api_button_openai
                        class={move || button_style_classes(ApiProvider::OpenAI)}>
                        OpenAI<br/>
                        <div class="font-mono">gpt{NBHY}4o</div>
                    </button>
                    <button
                        on:click=handle_api_button_claude
                        class={move || button_style_classes(ApiProvider::Claude)}>
                        Claude<br/>
                        <div class="font-mono">3.7{NBHY}sonnet</div>
                    </button>
                    <button
                        on:click=handle_api_button_gemini
                        class={move || button_style_classes(ApiProvider::Gemini)}>
                        Gemini<br/>
                        <div class="font-mono">2.0{NBHY}flash</div>
                    </button>
                    <button
                        on:click=handle_api_button_free
                        class={move || button_style_classes(ApiProvider::Free)}>
                        Free API<br/>
                        <div class="font-mono">limited</div>
                    </button>
                </div>

                {move || (api_provider.get() == ApiProvider::OpenAI).then(|| api_key_input_sec("sk-..."))}
                {move || (api_provider.get() == ApiProvider::Claude).then(|| api_key_input_sec("sk-..."))}
                {move || (api_provider.get() == ApiProvider::Gemini).then(|| api_key_input_sec("AI..."))}
                {move || (api_provider.get() == ApiProvider::Free).then(api_key_free_choice)}
            </div>
        }
    };

    // coolapsed view when this step has been completed
    let collapsed_view = move || {
        view! {
            <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm">
                <div class="absolute -top-3 -left-5 px-4 py-2 bg-gray-500 rounded-full flex items-center justify-center text-lg text-white font-semibold">
                    Step 1
                </div>

                <div class="text-center text-gray-800 text-lg">
                    <span class="font-semibold">API Provider and Model:{NBSP}{NBSP}</span>
                    <span class="text-xl font-mono">{move || api_provider.get().name()}</span>
                </div>

                {move || (api_provider.get() != ApiProvider::Null).then(|| view! {
                    <button
                        on:click=handle_back_button
                        class="absolute -bottom-3 -right-5 px-4 py-2 bg-gray-500 hover:bg-gray-600 rounded-md flex items-center justify-center text-white transition-colors">
                        Back
                    </button>
                })}
            </div>
        }
    };

    view! {
        {move || (stage.get() == Stage::Initial).then(expanded_view)}
        {move || (stage.get() > Stage::Initial).then(collapsed_view)}
    }
}
