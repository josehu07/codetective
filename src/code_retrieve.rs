//! Step 2 section: import/retrieve code for analysis

use leptos::prelude::*;

use crate::apis::ApiClient;
use crate::utils::gadgets::{StepHeaderCollapsed, StepHeaderExpanded};
use crate::Stage;

#[component]
pub(crate) fn CodeRetrieve(
    api_client: ReadSignal<Option<ApiClient>>,
    stage: ReadSignal<Stage>,
    set_stage: WriteSignal<Stage>,
) -> impl IntoView {
    // for the back button functionality
    // let handle_back_button = move |_| {
    //     set_stage.set(Stage::Initial);
    // };

    // expanded view when this step is active
    let expanded_view = move || {
        view! {
            <div class="relative max-w-4xl w-full mt-14 px-8 py-6 bg-white/60 rounded-lg shadow-sm animate-fade-in">
                <StepHeaderExpanded step=2 />

                <div class="text-xl text-center text-gray-900">Import Code for Analysis...</div>

            </div>
        }
    };

    // collapsed view when this step has been completed
    let collapsed_view = move || {
        view! {
            <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm">
                <StepHeaderCollapsed step=2 />

            // <div class="text-center text-gray-800 text-lg">
            // <span class="font-semibold">API Provider and Model:{NBSP}{NBSP}</span>
            // <span class="text-xl font-mono">{move || api_provider.get().name()}</span>
            // </div>

            // {move || {
            // (api_provider.get() != ApiProvider::Null)
            // .then(|| {
            // view! {
            // <button
            // on:click=handle_back_button
            // class="absolute -bottom-3 -right-5 px-4 py-2 bg-gray-500 hover:bg-gray-600 rounded-md flex items-center justify-center text-white transition-colors"
            // >
            // Back
            // </button>
            // }
            // })
            // }}
            </div>
        }
    };

    view! {
        {move || (stage.get() == Stage::ApiProvided).then(expanded_view)}
        {move || (stage.get() > Stage::ApiProvided).then(collapsed_view)}
    }
}
