//! Step 2 section: pload or paste code for analysis

use leptos::prelude::*;

use super::{Stage, NBSP};
use crate::api_selection::ApiProvider;

#[component]
pub(crate) fn CodeAnalysis(
    api_provider: ReadSignal<ApiProvider>,
    api_key: ReadSignal<String>,
    stage: ReadSignal<Stage>,
    set_stage: WriteSignal<Stage>,
) -> impl IntoView {
    // for the back button functionality
    let handle_back_button = move |_| {
        set_stage.set(Stage::Initial);
    };

    // expanded view when this step is active
    let expanded_view = move || {
        view! {
            <div class="relative max-w-4xl w-full mt-14 px-8 py-6 bg-white/60 rounded-lg shadow-sm animate-fade-in">
                <div class="absolute -top-3 -left-5 px-4 py-2 bg-gray-600 rounded-full flex items-center justify-center text-xl text-white font-semibold">
                    Step 2
                </div>

                <div class="text-xl text-center text-gray-900 mb-4">Add Code for Analysis</div>

                <div class="flex flex-col space-y-4">
                    <div class="flex space-x-4 justify-center">
                        <button class="px-5 py-3 bg-gray-500 hover:bg-gray-600 text-white rounded-md shadow transition-colors flex items-center">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM6.293 6.707a1 1 0 010-1.414l3-3a1 1 0 011.414 0l3 3a1 1 0 01-1.414 1.414L11 5.414V13a1 1 0 11-2 0V5.414L7.707 6.707a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                            </svg>
                            Upload File
                        </button>
                        <button class="px-5 py-3 bg-gray-500 hover:bg-gray-600 text-white rounded-md shadow transition-colors flex items-center">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                <path d="M8 2a1 1 0 000 2h2a1 1 0 100-2H8z" />
                                <path d="M3 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v6h-4.586l1.293-1.293a1 1 0 00-1.414-1.414l-3 3a1 1 0 000 1.414l3 3a1 1 0 001.414-1.414L10.414 13H15v3a2 2 0 01-2 2H5a2 2 0 01-2-2V5zM15 11h2a1 1 0 110 2h-2v-2z" />
                            </svg>
                            Paste Code
                        </button>
                    </div>

                    <div class="p-4 border-2 border-dashed border-gray-300 rounded-lg bg-white/80 min-h-[200px] flex items-center justify-center">
                        <div class="text-gray-500 text-center">
                            Drag and drop your code file here, or click one of the buttons above to begin.
                        </div>
                    </div>
                </div>
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
        {move || (stage.get() == Stage::ApiProvided).then(expanded_view)}
        {move || (stage.get() > Stage::ApiProvided).then(collapsed_view)}
    }
}
