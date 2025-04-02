//! Step 3 section: make the AI authorship detection pass

use leptos::prelude::*;
use leptos::task::spawn_local;

use gloo_timers::future::TimeoutFuture;

use crate::apis::ApiClient;
use crate::file::{CodeFile, CodeGroup, MAX_FILE_SIZE, MAX_NUM_FILES};
use crate::utils::error::CodeImportError;
use crate::utils::gadgets::{
    FailureIndicator, HoverInfoIcon, InvisibleIndicator, SpinningIndicator, StepHeaderCollapsed,
    StepHeaderExpanded, SuccessIndicator,
};
use crate::utils::NBSP;
use crate::{Stage, ValidationState};

// Helper functions and handler "closure"s:
fn format_file_size(size_opt: Option<usize>) -> String {
    match size_opt {
        None => "-".to_string(),
        Some(bytes) => {
            if bytes < 1024 {
                format!("{}", bytes)
            } else {
                format!("{:.2} KB", (bytes as f64) / 1024.0)
            }
        }
    }
}

// Different display components shown selectively:
#[component]
fn DetectionPassExpandedView(
    api_client: RwSignal<Option<ApiClient>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<Stage>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm animate-fade-in">
            <StepHeaderExpanded step=3 />

            <div class="text-xl text-center text-gray-900">Let the Analysis Begin...</div>

            <div class="mt-6 overflow-x-auto">
                <table class="min-w-full bg-white rounded-lg overflow-hidden">
                    <thead class="bg-gray-50">
                        <tr>
                            <th class="px-4 py-2 text-left text-base font-medium text-gray-700">
                                Code File
                            </th>
                            <th class="px-4 py-2 text-right text-base font-medium text-gray-700">
                                Language
                            </th>
                            <th class="px-4 py-2 text-right text-base font-medium text-gray-700">
                                Size
                            </th>
                            <th class="px-4 py-2 text-center text-base font-medium text-gray-700">
                                Status
                            </th>
                        </tr>
                    </thead>

                    <tbody>
                        <For
                            each=move || code_group.read().sorted_files()
                            key=|(path, _)| path.clone()
                            let((path, file))
                        >
                            <tr class="border-t border-gray-200 hover:bg-gray-50 transition-colors duration-50">
                                <td class="px-4 py-2 text-base text-gray-900 text-left font-mono">
                                    {CodeFile::path_display(path.as_str())}
                                </td>
                                <td class="px-4 py-2 text-sm text-gray-800 text-right">
                                    {CodeFile::lang_name_of(file.read().get_ext())}
                                </td>
                                <td class="px-4 py-2 text-sm text-gray-800 text-right">
                                    {format_file_size(file.read().get_size())}
                                </td>
                                <td class="px-4 py-2 flex justify-center text-center">
                                    <SpinningIndicator />
                                </td>
                            </tr>
                        </For>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

/// The code retrieval step wrapped in one place.
#[component]
pub(crate) fn DetectionPass(
    api_client: RwSignal<Option<ApiClient>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<Stage>,
) -> impl IntoView {
    view! {
        {move || {
            (stage.get() == Stage::CodeImported)
                .then_some(view! { <DetectionPassExpandedView api_client code_group stage /> })
        }}
    }
}
