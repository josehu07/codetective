//! Step 3 section: make the AI authorship detection pass

use std::collections::VecDeque;

use leptos::prelude::*;

use gloo_timers::future::TimeoutFuture;

use crate::apis::ApiClient;
use crate::file::{CodeFile, CodeGroup};
use crate::utils::gadgets::{
    BlinkDotsIndicator, FailureIndicator, HoverResultDiv, SpinningIndicator, StepHeaderExpanded,
    SuccessIndicator,
};
use crate::StepStage;

/// Time-wise spacing between task queue pollinngs.
const TASK_POLLING_DELAY: u32 = 1000; // 1 sec

/// Represents the status of a file's detection progress.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DetectionStatus {
    Pending,
    Flying,
    Success((u8, String)), // percentage of AI authorship and reasoning
    Failure(String),       // error message
}

/// Type alias for the analysis task queue.
pub(crate) type TaskQueue = VecDeque<(String, RwSignal<CodeFile>, RwSignal<DetectionStatus>)>;

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

pub(crate) async fn detection_analysis_task(
    api_client: RwSignal<Option<ApiClient>>,
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
) {
    // this task never terminates
    loop {
        // wakes up every such interval to grab a new task if any
        // this waiting also serves the purpose of rate limiting to make LLM
        // APIs happy
        TimeoutFuture::new(TASK_POLLING_DELAY).await;
        let next_task = task_queue.try_update(|queue| queue.pop_front());

        if let Some(Some((path, file, status))) = next_task {
            status.set(DetectionStatus::Flying);
            log::info!("Step 3 analyzing file '{}'...", path);

            if let Some(client) = api_client.read_untracked().as_ref() {
                match file
                    .read_untracked()
                    .content(code_group.read_untracked().cg_client())
                    .await
                {
                    Ok(content) => {
                        status.set(DetectionStatus::Success((
                            (getrandom::u32().unwrap() % 101) as u8,
                            "Some random reasoning".to_string(),
                        )));
                    }
                    Err(err) => {
                        log::error!("Analysis of file '{}' failed: {}", path, err);
                        status.set(DetectionStatus::Failure(err.to_string()));
                    }
                }
            }

            // update the number of finished tasks
            num_finished.update(|num| *num += 1);
            if num_finished.get_untracked() >= code_group.read_untracked().num_files() {
                // done with all tasks for now, if not rolling back
                log::info!("Step 3 detection analysis all tasks completed");
                detection_cp.set(true);
            }
        }
    }
}

// Display components of the detection analysis table:
#[component]
fn FileDetectionRow(
    path: String,
    file: RwSignal<CodeFile>,
    task_queue: RwSignal<TaskQueue>,
    detection_cp: RwSignal<bool>,
) -> impl IntoView {
    let detect_status = RwSignal::new(DetectionStatus::Pending);

    // queue this file for processing upon load of row
    if !detection_cp.get_untracked() {
        task_queue.update(|queue| {
            queue.push_back((path.clone(), file, detect_status));
        });
    }

    view! {
        <tr class="border-t border-gray-200 hover:bg-gray-50 transition-colors duration-50">
            <td class="px-4 py-2 w-96 text-base text-gray-900 text-left font-mono">
                {move || CodeFile::path_display(path.as_str())}
            </td>
            <td class="px-4 py-2 w-32 text-sm text-gray-800 text-right">
                {move || CodeFile::lang_name_of(file.read().get_ext())}
            </td>
            <td class="px-4 py-2 w-28 text-sm text-gray-800 text-right">
                {move || format_file_size(file.read().get_size())}
            </td>

            <td class="px-4 py-2 w-24 text-sm text-center">
                <div class="flex justify-center">
                    {move || {
                        matches!(*detect_status.read(), DetectionStatus::Pending)
                            .then_some(view! { <SpinningIndicator /> })
                    }}
                    {move || {
                        matches!(*detect_status.read(), DetectionStatus::Flying)
                            .then_some(view! { <SpinningIndicator /> })
                    }}
                    {move || {
                        matches!(*detect_status.read(), DetectionStatus::Success(_))
                            .then_some(view! { <SuccessIndicator /> })
                    }}
                    {move || {
                        matches!(*detect_status.read(), DetectionStatus::Failure(_))
                            .then_some(view! { <FailureIndicator /> })
                    }}
                </div>
            </td>

            <td class="px-4 py-2 w-auto text-sm text-center">
                <div class="flex justify-center">
                    {move || {
                        matches!(*detect_status.read(), DetectionStatus::Flying)
                            .then_some(view! { <BlinkDotsIndicator /> })
                    }}
                    {move || {
                        if let DetectionStatus::Success((percent, reason)) = detect_status.get() {
                            Some(view! { <HoverResultDiv percent=Some(percent) message=reason /> })
                        } else {
                            None
                        }
                    }}
                    {move || {
                        if let DetectionStatus::Failure(err_msg) = detect_status.get() {
                            Some(view! { <HoverResultDiv percent=None message=err_msg /> })
                        } else {
                            None
                        }
                    }}
                </div>
            </td>
        </tr>
    }
}

#[component]
fn DetectionPassExpandedView(
    api_client: RwSignal<Option<ApiClient>>,
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    detection_cp: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm animate-fade-in">
            <StepHeaderExpanded step=3 />

            {move || {
                if detection_cp.get() {
                    view! {
                        <div class="text-lg font-semibold text-center text-gray-900 animate-slide-down">
                            AI Authorship Detection Completed
                        </div>
                    }
                } else {
                    view! {
                        <div class="text-xl text-center text-gray-900">
                            Let the Analysis Begin...
                        </div>
                    }
                }
            }}

            <div class="mt-6 mb-2 overflow-x-auto">
                <table class="min-w-full bg-white rounded-lg overflow-hidden">
                    <thead class="bg-gray-50">
                        <tr>
                            <th class="px-4 py-2 w-96 text-left text-base font-medium text-gray-700">
                                Code File
                            </th>
                            <th class="px-4 py-2 w-32 text-right text-base font-medium text-gray-700">
                                Language
                            </th>
                            <th class="px-4 py-2 w-28 text-right text-base font-medium text-gray-700">
                                Size
                            </th>
                            <th class="px-4 py-2 w-24 text-center text-base font-medium text-gray-700">
                                Status
                            </th>
                            <th class="px-4 py-2 w-auto text-center text-base font-medium text-gray-700">
                                Result
                            </th>
                        </tr>
                    </thead>

                    <tbody>
                        <For
                            each=move || code_group.read().sorted_files()
                            key=|(path, _)| path.clone()
                            let((path, file))
                        >
                            <FileDetectionRow path file task_queue detection_cp />
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
    task_queue: RwSignal<TaskQueue>,
    detection_cp: RwSignal<bool>,
    stage: RwSignal<StepStage>,
) -> impl IntoView {
    view! {
        {move || {
            (stage.get() >= StepStage::CodeGot)
                .then_some(
                    view! {
                        <DetectionPassExpandedView api_client code_group task_queue detection_cp />
                    },
                )
        }}
    }
}
