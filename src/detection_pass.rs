//! Step 3 section: make the AI authorship detection pass

use std::cmp;
use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use reqwest::Client as CgfClient;

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use web_sys::HtmlAnchorElement;

use gloo_timers::future::TimeoutFuture;

use gloo_file::{Blob, ObjectUrl};

use crate::apis::ApiClient;
use crate::file::{CodeFile, CodeGroup};
use crate::utils::error::ApiMakeCallError;
use crate::utils::gadgets::{
    BlinkDotsIndicator, FailureIndicator, HoverInfoIcon, HoverResultDiv, SpinningIndicator,
    StepHeaderExpanded, SuccessIndicator,
};
use crate::{StepStage, NBSP};

/// Time-wise spacing between task queue pollinngs.
const TASK_POLLING_DELAY: u32 = 1000; // 1 sec

/// File name to propose when downloading the results.
const DOWNLOAD_FILENAME: &str = "codetective_results.json";

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

/// Type alias for the file results map.
pub(crate) type FileResults = Vec<(String, RwSignal<CodeFile>, RwSignal<DetectionStatus>)>;

/// Helper structs for putting the analysis results together for JSON downloading.
#[derive(Serialize, Deserialize, Debug)]
struct DownloadableResults {
    results: Vec<DownloadableResultsEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DownloadableResultsEntry {
    file: String,
    lang: String,
    size: Option<usize>,
    finished: bool,
    likelihood: Option<u8>,
    reasoning: Option<String>,
    error_msg: Option<String>,
}

impl DownloadableResults {
    fn from(file_results: &FileResults) -> Self {
        let mut results = Vec::new();
        for (path, code_file, detect_status) in file_results.iter() {
            let file = path.clone();
            let lang = CodeFile::lang_name_of(code_file.read().get_ext());
            let size = code_file.read().get_size();
            let status = detect_status.get();
            let finished = matches!(status, DetectionStatus::Success(_));
            let likelihood = match status {
                DetectionStatus::Success((percent, _)) => Some(percent),
                _ => None,
            };
            let (reasoning, error_msg) = match status {
                DetectionStatus::Success((_, reason)) => (Some(reason), None),
                DetectionStatus::Failure(err_msg) => (None, Some(err_msg)),
                _ => (None, Some("Analysis for this file is still in progress (which generally should not happend at the time of download).".to_string())),
            };

            results.push(DownloadableResultsEntry {
                file,
                lang,
                size,
                finished,
                likelihood,
                reasoning,
                error_msg,
            });
        }

        DownloadableResults { results }
    }
}

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

async fn detection_api_call(client: &ApiClient, code: &str) -> DetectionStatus {
    match client.call(code).await {
        Ok((percent, reason)) => DetectionStatus::Success((percent, format!("Reasoning: {}", reason))),
        Err(err) => DetectionStatus::Failure(match err {
            ApiMakeCallError::Parse(_) => "Failed to parse API response. This could be due to unexpected model output format or truncation (despite being instructed otherwise), or due to rate limiting. Please try again later.",
            ApiMakeCallError::Status(_) => "Network error when making the API call. This could be due to connection issues, model unavailability, authorization failure, or mostly likely, rate limiting. Please try again later.",
        }.to_string()),
    }
}

pub(crate) async fn detection_analysis_task(
    api_client: RwSignal<Option<ApiClient>>,
    cgf_client: RwSignal<CgfClient>,
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
    stage: RwSignal<StepStage>,
) {
    // this task never terminates
    loop {
        // wakes up every such interval to grab a new task if any
        // this waiting also serves the purpose of rate limiting to make LLM
        // APIs happy
        TimeoutFuture::new(TASK_POLLING_DELAY).await;
        let next_task = task_queue.try_update(|queue| queue.pop_front());

        if let Some(Some((path, file, status))) = next_task {
            // got task for file
            status.set(DetectionStatus::Flying);
            log::info!("Step 3 analyzing file '{}'...", path);

            // take the client out in each iteration, to avoid holding a guard
            // to the signal while awaiting; otherwise, the back buttons might
            // trigger panics
            let api_client_taken = api_client.write().take();
            if let Some(client) = api_client_taken {
                match file
                    .read_untracked()
                    .content(&cgf_client.read_untracked())
                    .await
                {
                    Ok(code) => {
                        status.set(detection_api_call(&client, &code).await);
                    }
                    Err(err) => {
                        log::error!("Analysis of file '{}' failed: {}", path, err);
                        status.set(DetectionStatus::Failure(err.to_string()));
                    }
                }

                let now_stage = stage.get_untracked();
                if now_stage >= StepStage::ApiDone {
                    // put client back
                    api_client.try_update(|api_client| {
                        if api_client.is_none() {
                            *api_client = Some(client);
                        }
                    });

                    if now_stage == StepStage::CodeGot {
                        // update num_finished counter
                        num_finished.update(|num| *num += 1);
                        if num_finished.get_untracked() >= code_group.read_untracked().num_files() {
                            // done with all tasks for now, if not rolling back
                            log::info!("Step 3 detection analysis all tasks completed");
                            detection_cp.set(true);
                        }
                    }
                }
            } else {
                status.set(DetectionStatus::Failure("API client not available. There seems to be an internal error; please refresh the page.".to_string()));
            }
        } else {
            // no task in the queue, do nothing, go sleep again
        }
    }
}

fn handle_retry_button(
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
    file_results: RwSignal<FileResults>,
    nothing_to_retry: RwSignal<bool>,
) {
    // put all failed tasks back to the queue, in sorted order
    let mut num_retried = 0;
    for (path, file, detect_status) in file_results.read().iter() {
        if matches!(*detect_status.read(), DetectionStatus::Failure(_)) {
            detect_status.set(DetectionStatus::Pending);
            task_queue.update(|queue| {
                queue.push_back((path.clone(), *file, *detect_status));
            });
            num_retried += 1;
        }
    }

    if num_retried > 0 {
        nothing_to_retry.set(false);
        num_finished.update(|num| *num -= cmp::min(num_retried, *num));
        detection_cp.set(false);
    } else {
        nothing_to_retry.set(true);
    }
}

fn handle_download_button(file_results: RwSignal<FileResults>) {
    let results = DownloadableResults::from(&file_results.read());
    match serde_json::to_string_pretty(&results) {
        Ok(results_json) => {
            let blob = Blob::new(results_json.as_str());
            let url = ObjectUrl::from(blob);

            // create an invisible download link
            let document = web_sys::window()
                .expect("No window found in the DOM")
                .document()
                .expect("No document found in the DOM");

            let a = document
                .create_element("a")
                .expect("Failed to create anchor element for download")
                .dyn_into::<HtmlAnchorElement>()
                .expect("Failed to cast anchor element type for download");
            a.set_href(&url);
            a.set_download("codetective_results.json");
            a.style(("display", "none"));

            // add to body, click to trigger download, then remove
            log::info!("Downloading results as '{}'...", DOWNLOAD_FILENAME);
            document
                .body()
                .expect("No body found in the DOM")
                .append_child(&a)
                .expect("Failed to append anchor element to body for download");
            a.click();
            document
                .body()
                .expect("No body found in the DOM")
                .remove_child(&a)
                .expect("Failed to remove anchor element from body after download");
        }

        Err(err) => {
            log::error!("Failed to serialize results to JSON: {}", err);
            // ignore and let users redo
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
    file_results: RwSignal<FileResults>,
) -> impl IntoView {
    let detect_status = RwSignal::new(DetectionStatus::Pending);

    // queue this file for processing upon load of row
    if !detection_cp.get_untracked() {
        task_queue.update(|queue| {
            queue.push_back((path.clone(), file, detect_status));
        });
        file_results.update(|results| {
            results.push((path.clone(), file, detect_status));
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
fn NoRetryErrorMsg(nothing_to_retry: RwSignal<bool>) -> impl IntoView {
    move || {
        if nothing_to_retry.get() {
            Some(view! {
                <div class="text-red-700 text-base font-mono mt-4 text-center animate-fade-in">
                    Nothing to retry, all files have been analyzed successfully
                </div>
            })
        } else {
            None
        }
    }
}

#[component]
fn DetectionPassExpandedView(
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
    file_results: RwSignal<FileResults>,
    nothing_to_retry: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm animate-fade-in">
            <StepHeaderExpanded step=3 />

            {move || {
                (detection_cp.get())
                    .then_some(
                        view! {
                            <div class="text-center text-gray-800 text-lg animate-slide-down">
                                <span class="font-semibold">
                                    AI Authorship Analyzed:{NBSP}{NBSP}
                                </span>
                                <span class="text-xl font-mono">See Likelihood Results</span>
                            </div>
                        },
                    )
            }}
            {move || {
                (!detection_cp.get())
                    .then_some(
                        view! {
                            <div class="text-xl text-center text-gray-900">
                                Let the Analysis Begin...
                            </div>
                        },
                    )
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
                            <FileDetectionRow path file task_queue detection_cp file_results />
                        </For>
                    </tbody>
                </table>
            </div>

            {move || {
                (detection_cp.get())
                    .then_some(
                        view! {
                            <div class="mt-6 mb-2 flex items-center justify-center space-x-8 w-full animate-slide-down">
                                <button
                                    on:click=move |_| handle_retry_button(
                                        task_queue,
                                        num_finished,
                                        detection_cp,
                                        file_results,
                                        nothing_to_retry,
                                    )
                                    class="px-4 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-md shadow transition-colors flex align-middle"
                                >
                                    Retry
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="inline w-5 h-5 ml-2 my-auto"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                                        />
                                    </svg>
                                </button>

                                <button
                                    on:click=move |_| handle_download_button(file_results)
                                    class="px-4 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-md shadow transition-colors flex align-middle"
                                >
                                    Download
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="inline w-5 h-5 ml-2 my-auto"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                                        />
                                    </svg>
                                </button>

                                <div class="absolute right-0 z-20">
                                    <HoverInfoIcon text="Don't fully trust the likelihood scores as they can be deceiving: oftentimes, well-written code by human would be categorized as AI-generated as they follow good coding standards. Different language models may also produce undeniably different scores. Be sure to read the reasoning comments and make your own judgement." />
                                </div>
                            </div>
                        },
                    )
            }}

            <NoRetryErrorMsg nothing_to_retry />
        </div>
    }
}

/// The code retrieval step wrapped in one place.
#[component]
pub(crate) fn DetectionPass(
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
    file_results: RwSignal<FileResults>,
    nothing_to_retry: RwSignal<bool>,
    stage: RwSignal<StepStage>,
) -> impl IntoView {
    view! {
        {move || {
            (stage.get() >= StepStage::CodeGot)
                .then_some(
                    view! {
                        <DetectionPassExpandedView
                            code_group
                            task_queue
                            num_finished
                            detection_cp
                            file_results
                            nothing_to_retry
                        />
                    },
                )
        }}
    }
}
