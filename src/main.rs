//! Main entrance to the codetective web app.

use std::collections::VecDeque;

use reqwest::Client as CgfClient;

use leptos::prelude::*;
use leptos::task::spawn_local;

use leptos_meta::{provide_meta_context, Title};

pub(crate) mod api_selection;
use api_selection::{ApiProvider, ApiSelection};

pub(crate) mod code_retrieve;
use code_retrieve::{CodeRetrieve, ImportMethod};

pub(crate) mod detection_pass;
use detection_pass::{detection_analysis_task, DetectionPass, FileResults, TaskQueue};

pub(crate) mod apis;

pub(crate) mod file;
use file::CodeGroup;

pub(crate) mod utils;
use utils::gadgets::GitHubBanner;
use utils::NBSP;

/// Stage enum that controls where are we in the workflow.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum StepStage {
    Initial,
    ApiDone,
    CodeGot,
}

/// Step-generic input validation state.
#[derive(Clone, PartialEq, Debug)]
enum ValidationState<E> {
    Idle,
    Pending,
    Success,
    Failure(E),
}

/// Currently, the app only has one route, which is the home page.
#[component]
fn Home() -> impl IntoView {
    let stage = RwSignal::new(StepStage::Initial);

    let api_client = RwSignal::new(None);
    let cgf_client = RwSignal::new(CgfClient::new());
    let code_group = RwSignal::new(CodeGroup::new());

    let api_provider = RwSignal::new(ApiProvider::Null);
    let input_api_key = RwSignal::new(String::new());
    let api_key_vstate = RwSignal::new(ValidationState::Idle);

    let import_method = RwSignal::new(ImportMethod::Null);
    let input_code_url = RwSignal::new(String::new());
    let input_code_text = RwSignal::new(String::new());
    let code_in_vstate = RwSignal::new(ValidationState::Idle);

    let task_queue = RwSignal::new(VecDeque::new());
    let num_finished = RwSignal::new(0);
    let detection_cp = RwSignal::new(false);
    let file_results = RwSignal::new(Vec::new());

    // spawn the detection analysis task ahead of time, which periodically
    // polls the task queue
    spawn_local(async move {
        log::debug!("Detection analysis task created and polling...");
        detection_analysis_task(
            api_client,
            cgf_client,
            code_group,
            task_queue,
            num_finished,
            detection_cp,
            stage,
        )
        .await;
    });

    view! {
        <Title text="Codetective" />
        <main>
            <div class="bg-gradient-to-tl from-gray-300 to-gray-200 text-black font-sans flex flex-col max-w-full min-h-screen">
                // main body sections
                <div class="flex flex-col items-center pt-10">
                    // title and logo
                    <div class="flex flex-col items-center">
                        <div class="flex items-end justify-center">
                            <h1 class="text-5xl font-bold text-gray-600">Co</h1>
                            <h1 class="text-5xl font-bold text-gray-900">de</h1>
                            <h1 class="text-5xl font-bold text-gray-600">tective</h1>
                            <img src="./codetective.png" alt="Codetective Logo" class="ml-4 h-16" />
                        </div>
                        <h2 class="text-2xl font-semibold text-gray-600 mt-4">
                            Code AI Authorship Detection in 5 Clicks
                        </h2>
                    </div>

                    // step 1:
                    <ApiSelection
                        api_provider
                        input_api_key
                        api_key_vstate
                        api_client
                        code_in_vstate
                        code_group
                        task_queue
                        num_finished
                        detection_cp
                        file_results
                        stage
                    />

                    // step 2:
                    <CodeRetrieve
                        import_method
                        input_code_url
                        input_code_text
                        code_in_vstate
                        cgf_client
                        code_group
                        task_queue
                        num_finished
                        detection_cp
                        file_results
                        stage
                    />

                    // step 3:
                    <DetectionPass
                        code_group
                        task_queue
                        num_finished
                        detection_cp
                        file_results
                        stage
                    />
                </div>

                // footer text and links
                <footer class="mt-auto pb-6 pt-8 flex text-center justify-center">
                    <span class="mr-3">
                        <a
                            href="https://github.com/josehu07/codetective"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            <GitHubBanner />
                        </a>
                    </span>
                    <p class="text-sm text-gray-500">
                        Made with {NBSP}
                        <a
                            href="https://leptos.dev"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-blue-700 hover:underline"
                        >
                            Rust Leptos
                        </a> {NBSP}+ {NBSP}
                        <a
                            href="https://tailwindcss.com"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-blue-700 hover:underline"
                        >
                            Tailwind CSS
                        </a> {NBSP}+ {NBSP}
                        <a
                            href="https://trunkrs.dev"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-blue-700 hover:underline"
                        >
                            Trunk WASM
                        </a>. {NBSP}{NBSP}Authored by {NBSP}
                        <a
                            href="https://josehu.com"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-blue-700 hover:underline"
                        >
                            Guanzhou Hu
                        </a>.
                    </p>
                </footer>
            </div>
        </main>
    }
}

/// The main entry point to the application.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! { <Home /> }
}

fn main() {
    _ = console_log::init_with_level(log::Level::Info);
    console_error_panic_hook::set_once();

    mount_to_body(App)
}
