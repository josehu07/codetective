//! Step 2 section: import/retrieve code for analysis

use leptos::prelude::*;

use crate::apis::ApiClient;
use crate::utils::gadgets::{button_style_classes, StepHeaderCollapsed, StepHeaderExpanded};
use crate::utils::NBSP;
use crate::Stage;

/// Enum that controls the state of code retrieval method selection.
#[derive(Clone, Copy, PartialEq, Debug)]
enum ImportMethod {
    UrlTo,
    Upload,
    Paste,
    Null,
}

// Helper functions and handler "closure"s:
fn handle_import_method_button(
    set_import_method: WriteSignal<ImportMethod>,
    selected_method: ImportMethod,
) {
    set_import_method.set(selected_method);
}

// Different display components shown selectively:
#[component]
fn CodeRetrieveExpandedView(
    import_method: ReadSignal<ImportMethod>,
    set_import_method: WriteSignal<ImportMethod>,
    set_stage: WriteSignal<Stage>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm animate-fade-in">
            <StepHeaderExpanded step=2 />

            <div class="text-xl text-center text-gray-900">Import Code for Analysis...</div>

            <div class="flex space-x-6 mt-6 mb-2 justify-center">
                <button
                    on:click=move |_| handle_import_method_button(set_import_method, ImportMethod::UrlTo)
                    class=move || button_style_classes(import_method.get() == ImportMethod::UrlTo)
                >
                    URL to
                    <br />
                    <span class="font-mono">file</span>{NBSP}or{NBSP}<span class="font-mono">repo</span>
                </button>
                <button
                    on:click=move |_| handle_import_method_button(set_import_method, ImportMethod::Upload)
                    class=move || button_style_classes(import_method.get() == ImportMethod::Upload)
                >
                    Upload
                    <br />
                    <span class="font-mono">file</span>{NBSP}or{NBSP}<span class="font-mono">zip</span>
                </button>
                <button
                    on:click=move |_| handle_import_method_button(set_import_method, ImportMethod::Paste)
                    class=move || button_style_classes(import_method.get() == ImportMethod::Paste)
                >
                    Paste to
                    <br />
                    <span class="font-mono">textbox</span>
                </button>
            </div>
        </div>
    }
}

#[component]
fn CodeRetrieveCollapsedView(set_stage: WriteSignal<Stage>) -> impl IntoView {
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
}

/// The code retrieval step wrapped in one place.
#[component]
pub(crate) fn CodeRetrieve(
    stage: ReadSignal<Stage>,
    set_stage: WriteSignal<Stage>,
) -> impl IntoView {
    let (import_method, set_import_method) = signal(ImportMethod::Null);

    view! {
        {move || (stage.get() == Stage::ApiProvided).then_some(view! {
            <CodeRetrieveExpandedView
                import_method
                set_import_method
                set_stage
            />
        })}

        {move || (stage.get() > Stage::ApiProvided).then_some(view! {
            <CodeRetrieveCollapsedView
                set_stage
            />
        })}
    }
}
