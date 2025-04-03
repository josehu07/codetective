//! Step 2 section: import/retrieve code for analysis

use leptos::prelude::*;
use leptos::task::spawn_local;

use web_sys::DragEvent;

use gloo_file::FileList;
use gloo_timers::future::TimeoutFuture;

use crate::file::{CodeGroup, MAX_FILE_SIZE, MAX_NUM_FILES};
use crate::utils::error::CodeImportError;
use crate::utils::gadgets::{
    FailureIndicator, HoverInfoIcon, InvisibleIndicator, SpinningIndicator, StepHeaderCollapsed,
    StepHeaderExpanded, SuccessIndicator,
};
use crate::utils::NBSP;
use crate::{StepStage, TaskQueue, ValidationState};

/// Enum that controls the state of code retrieval method selection.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum ImportMethod {
    UrlTo,
    Upload,
    Paste,
    Null,
}

impl ImportMethod {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            ImportMethod::UrlTo => "URL",
            ImportMethod::Upload => "Upload",
            ImportMethod::Paste => "Textbox",
            ImportMethod::Null => "Null",
        }
    }
}

// Helper functions and handler "closure"s:
fn button_style_classes(is_selected: bool) -> String {
    format!(
        "w-36 h-16 rounded-lg shadow-md hover:shadow-lg flex-col items-center justify-center font-semibold border {}",
        if is_selected { "bg-gray-200 text-gray-900 border-gray-400" } else { "bg-white hover:bg-gray-200 text-gray-600 hover:text-gray-900 border-gray-300" },
    )
}

fn handle_import_method_button(
    import_method: RwSignal<ImportMethod>,
    selected_method: ImportMethod,
) {
    import_method.set(selected_method);
}

fn handle_code_url_submit(
    import_method: RwSignal<ImportMethod>,
    input_code_url: RwSignal<String>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<StepStage>,
) {
    let current_import_method = import_method.get();
    let code_url = input_code_url.read().trim().to_string();

    if code_url.is_empty() || !code_url.is_ascii() {
        log::warn!("Code URL input field is empty or non-ASCII, please try again...");
        code_in_vstate.set(ValidationState::Failure(CodeImportError::ascii(
            "code URL input is empty or non-ASCII",
        )));
        return;
    }

    code_in_vstate.set(ValidationState::Pending);

    spawn_local(async move {
        log::info!(
            "Step 2 validating: importing from {} '{}'...",
            current_import_method.name(),
            code_url
        );

        let mut code_group_inner = code_group.write();
        match code_group_inner.import_remote(&code_url).await {
            Ok(()) => {
                code_in_vstate.set(ValidationState::Success);

                // small delay before proceeding to next stage
                TimeoutFuture::new(500).await;

                log::info!(
                    "Step 2 confirmed: imported {} file(s) from {}",
                    code_group_inner.num_files(),
                    current_import_method.name()
                );
                stage.set(StepStage::CodeGot);
            }

            Err(err) => {
                log::error!(
                    "Code import from {} failed: {}",
                    current_import_method.name(),
                    err
                );
                code_in_vstate.set(ValidationState::Failure(err));
            }
        }
    });
}

fn handle_code_text_submit(
    import_method: RwSignal<ImportMethod>,
    input_code_text: RwSignal<String>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<StepStage>,
) {
    let current_import_method = import_method.get();
    let code_text = input_code_text.read().trim().to_string();

    if code_text.is_empty() {
        log::warn!("Code textbox input is empty, please try again...");
        code_in_vstate.set(ValidationState::Failure(CodeImportError::parse(
            "code textbox input is empty",
        )));
        return;
    }

    code_in_vstate.set(ValidationState::Pending);

    spawn_local(async move {
        log::info!(
            "Step 2 validating: importing from {}, size {}...",
            current_import_method.name(),
            code_text.len()
        );

        let mut code_group_inner = code_group.write();
        match code_group_inner.import_textbox(code_text).await {
            Ok(()) => {
                code_in_vstate.set(ValidationState::Success);

                // small delay before proceeding to next stage
                TimeoutFuture::new(500).await;

                log::info!(
                    "Step 2 confirmed: imported {} file(s) from {}",
                    code_group_inner.num_files(),
                    current_import_method.name()
                );
                stage.set(StepStage::CodeGot);
            }

            Err(err) => {
                log::error!(
                    "Code import from {} failed: {}",
                    current_import_method.name(),
                    err
                );
                code_in_vstate.set(ValidationState::Failure(err));
            }
        }
    });
}

fn handle_code_files_upload(
    import_method: RwSignal<ImportMethod>,
    file_list: FileList,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<StepStage>,
) {
    let current_import_method = import_method.get();

    if file_list.is_empty() {
        log::warn!("Code file list uploaded is empty, please try again...");
        code_in_vstate.set(ValidationState::Failure(CodeImportError::parse(
            "code file list uploaded is empty",
        )));
        return;
    }

    code_in_vstate.set(ValidationState::Pending);

    spawn_local(async move {
        log::info!(
            "Step 2 validating: importing from {}, {} files...",
            current_import_method.name(),
            file_list.len()
        );

        let mut code_group_inner = code_group.write();
        match code_group_inner.import_upload(file_list).await {
            Ok(()) => {
                code_in_vstate.set(ValidationState::Success);

                // small delay before proceeding to next stage
                TimeoutFuture::new(500).await;

                log::info!(
                    "Step 2 confirmed: imported {} file(s) from {}",
                    code_group_inner.num_files(),
                    current_import_method.name()
                );
                stage.set(StepStage::CodeGot);
            }

            Err(err) => {
                log::error!(
                    "Code import from {} failed: {}",
                    current_import_method.name(),
                    err
                );
                code_in_vstate.set(ValidationState::Failure(err));
            }
        }
    });
}

fn handle_back_button(
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
    stage: RwSignal<StepStage>,
) {
    code_in_vstate.set(ValidationState::Idle);
    code_group.update(|cg| {
        cg.reset();
    });
    task_queue.update(|queue| {
        queue.clear();
    });
    num_finished.set(0);
    detection_cp.set(false);
    stage.set(StepStage::ApiDone);

    log::info!("Step 2 rolled back: resetting code import validation stage");
}

// Different display components shown selectively:
#[component]
fn ValidationIndicator(
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
) -> impl IntoView {
    move || match code_in_vstate.get() {
        ValidationState::Idle => InvisibleIndicator().into_any(),
        ValidationState::Pending => SpinningIndicator().into_any(),
        ValidationState::Success => SuccessIndicator().into_any(),
        ValidationState::Failure(_) => FailureIndicator().into_any(),
    }
}

#[component]
fn ValidationErrorMsg(code_in_vstate: RwSignal<ValidationState<CodeImportError>>) -> impl IntoView {
    move || {
        if let ValidationState::Failure(err) = code_in_vstate.get() {
            Some(view! {
                <div class="text-red-700 text-base font-mono mt-4 text-center animate-fade-in">
                    {format!(
                        "Code import failed: {}",
                        match &err {
                            CodeImportError::Parse(msg) => &msg,
                            CodeImportError::Exists(msg) => &msg,
                            CodeImportError::Exten(msg) => &msg,
                            CodeImportError::Status(_) => "request failure, invalid URL or CORS?",
                            CodeImportError::Limit(msg) => &msg,
                            CodeImportError::Ascii(_) => "please provide a legit input source...",
                            CodeImportError::GitHub(msg) => &msg,
                            CodeImportError::Upload(msg) => &msg,
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
fn ImportFromUrlToSection(
    import_method: RwSignal<ImportMethod>,
    input_code_url: RwSignal<String>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<StepStage>,
    placeholder: &'static str,
) -> impl IntoView {
    view! {
        <div class="pt-6 pb-2 px-2 overflow-hidden animate-slide-down origin-top">
            <div class="flex items-center justify-center space-x-4">
                <label for="code-url" class="text-base text-gray-900 whitespace-nowrap">
                    Enter URL:
                </label>
                <input
                    type="url"
                    id="code-url"
                    placeholder=placeholder
                    prop:value=move || input_code_url.get()
                    prop:disabled=move || code_in_vstate.get() == ValidationState::Pending
                    on:input=move |ev| {
                        input_code_url.set(event_target_value(&ev));
                    }
                    on:keydown=move |ev| {
                        if ev.key_code() != 0 && ev.key() == "Enter"
                            && code_in_vstate.get() != ValidationState::Pending
                            && code_in_vstate.get() != ValidationState::Success
                        {
                            handle_code_url_submit(
                                import_method,
                                input_code_url,
                                code_in_vstate,
                                code_group,
                                stage,
                            );
                        }
                    }
                    class="flex-1 p-2 max-w-xl border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                />

                <HoverInfoIcon text="A URL link to either a raw online file or a GitHub repository. Size per file limited to 100KB. Number of files (if repo) capped to 100 (but may improve later)." />

                <button
                    on:click=move |_| {
                        if code_in_vstate.get() != ValidationState::Pending
                            && code_in_vstate.get() != ValidationState::Success
                        {
                            handle_code_url_submit(
                                import_method,
                                input_code_url,
                                code_in_vstate,
                                code_group,
                                stage,
                            );
                        }
                    }
                    disabled=move || code_in_vstate.get() == ValidationState::Pending
                    class=move || {
                        let base = "px-4 py-2 bg-gray-500 text-white rounded-md shadow transition-colors";
                        match code_in_vstate.get() {
                            ValidationState::Pending => {
                                format!("{} opacity-75 cursor-not-allowed", base)
                            }
                            _ => format!("{} hover:bg-gray-600", base),
                        }
                    }
                >
                    Confirm
                </button>

                <ValidationIndicator code_in_vstate />
            </div>

            <ValidationErrorMsg code_in_vstate />
        </div>
    }
}

#[component]
fn ImportFromUploadSection(
    import_method: RwSignal<ImportMethod>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<StepStage>,
) -> impl IntoView {
    let is_dragging = RwSignal::new(false);
    let file_input_ref = NodeRef::new();

    view! {
        <div class="pt-6 pb-2 px-2 overflow-hidden animate-slide-down origin-top">
            <div class="flex flex-col items-center justify-center space-y-4">
                <div class="w-full flex items-center space-x-4">
                    <label for="file-upload" class="text-base text-gray-900 whitespace-nowrap">
                        Upload file(s) or an archive
                        {NBSP}
                        (zip, tar, tar.gz, 7z):
                    </label>
                    <div class="flex-1"></div>
                    <HoverInfoIcon text="Upload one or more code files, or a supported archive. Size per file limited to 100KB. Number of files (if archive) capped to 100 (but may improve later)." />
                </div>

                <div
                    class=move || {
                        let base = "w-full border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors flex flex-col items-center justify-center";
                        if is_dragging.get() {
                            format!("{} bg-blue-50 border-blue-400", base)
                        } else if code_in_vstate.get() == ValidationState::Pending {
                            format!("{} bg-gray-100 border-gray-300 cursor-not-allowed", base)
                        } else {
                            format!(
                                "{} bg-gray-50 border-gray-300 hover:border-gray-400 hover:bg-gray-100",
                                base,
                            )
                        }
                    }
                    on:dragover=move |ev| {
                        ev.prevent_default();
                        is_dragging.set(true);
                    }
                    on:dragleave=move |ev| {
                        ev.prevent_default();
                        is_dragging.set(false);
                    }
                    on:drop=move |ev: DragEvent| {
                        ev.prevent_default();
                        is_dragging.set(false);
                        if let Some(data_transfer) = ev.data_transfer() {
                            if let Some(file_list) = data_transfer.files() {
                                handle_code_files_upload(
                                    import_method,
                                    file_list.into(),
                                    code_in_vstate,
                                    code_group,
                                    stage,
                                );
                            }
                        }
                    }
                    on:click=move |_| {
                        if code_in_vstate.get() != ValidationState::Pending {
                            if let Some(input) = file_input_ref.get() {
                                input.click();
                            }
                        }
                    }
                >
                    <input
                        type="file"
                        multiple
                        id="file-upload"
                        accept="*"
                        class="hidden"
                        node_ref=file_input_ref
                        on:change=move |_| {
                            if let Some(input) = file_input_ref.get() {
                                if let Some(file_list) = input.files() {
                                    handle_code_files_upload(
                                        import_method,
                                        file_list.into(),
                                        code_in_vstate,
                                        code_group,
                                        stage,
                                    );
                                }
                            }
                        }
                        on:click=move |ev| {
                            ev.stop_propagation();
                        }
                        prop:disabled=move || code_in_vstate.get() == ValidationState::Pending
                    />

                    <svg
                        class="w-12 h-12 text-gray-400 mb-3 pointer-events-none"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                        ></path>
                    </svg>

                    <div class="text-base font-medium text-gray-700 pointer-events-none">
                        {move || {
                            if is_dragging.get() {
                                "Drop file here..."
                            } else {
                                "Drag & drop file here, or Click to browse..."
                            }
                        }}
                    </div>
                </div>

                {move || {
                    (code_in_vstate.get() != ValidationState::Idle)
                        .then_some(
                            view! {
                                <div class="flex w-full items-center justify-end">
                                    <ValidationIndicator code_in_vstate />
                                </div>
                            },
                        )
                }}
            </div>

            <ValidationErrorMsg code_in_vstate />
        </div>
    }
}

#[component]
fn ImportFromPasteSection(
    import_method: RwSignal<ImportMethod>,
    input_code_text: RwSignal<String>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<StepStage>,
    placeholder: &'static str,
) -> impl IntoView {
    view! {
        <div class="pt-6 pb-2 px-2 overflow-hidden animate-slide-down origin-top">
            <div class="flex flex-col items-center justify-center space-y-4">
                <div class="w-full flex items-center space-x-4">
                    <label for="code-textbox" class="text-base text-gray-900 whitespace-nowrap">
                        Paste or type in code textbox directly:
                    </label>
                    <div class="flex-1"></div>
                    <HoverInfoIcon text="Paste or type your source code directly into the text box. Size limited to 100KB, but code files are normally much smaller than that." />
                </div>

                // wrap textarea in a div with almost identical styling so that
                // we can do the "hack" of using its height to auto-adjust the
                // height of the textarea. This trick is to make the wrapper div
                // invisible, and update its CSS "content" property to be the
                // real-time textarea value. Note that CSS strings use \A for
                // escaping newline
                <div
                    class="grid w-full after:min-h-[128px] after:p-3 after:border after:rounded after:font-mono after:text-sm after:whitespace-pre after:overflow-x-scroll after:invisible after:row-start-1 after:row-end-2 after:col-start-1 after:col-end-2 after:content-[attr(mirrored-content)]"
                    mirrored-content=move || format!("{} ", input_code_text.read())
                >
                    <textarea
                        id="code-textbox"
                        placeholder=placeholder
                        prop:value=move || input_code_text.get()
                        prop:disabled=move || code_in_vstate.get() == ValidationState::Pending
                        on:input=move |ev| {
                            input_code_text.set(event_target_value(&ev));
                        }
                        data-enable-grammarly="false"
                        class="w-full min-h-[128px] p-3 border border-gray-300 rounded font-mono text-sm whitespace-pre overflow-x-scroll focus:outline-none focus:ring-2 focus:ring-blue-500 overflow-hidden resize-none row-start-1 row-end-2 col-start-1 col-end-2"
                    />
                </div>

                <div class="flex items-center justify-end space-x-4 w-full">
                    <button
                        on:click=move |_| {
                            if code_in_vstate.get() != ValidationState::Pending
                                && code_in_vstate.get() != ValidationState::Success
                            {
                                handle_code_text_submit(
                                    import_method,
                                    input_code_text,
                                    code_in_vstate,
                                    code_group,
                                    stage,
                                );
                            }
                        }
                        disabled=move || code_in_vstate.get() == ValidationState::Pending
                        class=move || {
                            let base = "px-4 py-2 bg-gray-500 text-white rounded-md shadow transition-colors";
                            match code_in_vstate.get() {
                                ValidationState::Pending => {
                                    format!("{} opacity-75 cursor-not-allowed", base)
                                }
                                _ => format!("{} hover:bg-gray-600", base),
                            }
                        }
                    >
                        Confirm
                    </button>

                    <ValidationIndicator code_in_vstate />
                </div>
            </div>

            <ValidationErrorMsg code_in_vstate />
        </div>
    }
}

#[component]
fn CodeRetrieveExpandedView(
    import_method: RwSignal<ImportMethod>,
    input_code_url: RwSignal<String>,
    input_code_text: RwSignal<String>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<StepStage>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm animate-fade-in">
            <StepHeaderExpanded step=2 />

            <div class="text-xl text-center text-gray-900">Import Code for Analysis...</div>

            <div class="flex space-x-6 mt-6 mb-2 justify-center">
                <button
                    on:click=move |_| handle_import_method_button(
                        import_method,
                        ImportMethod::UrlTo,
                    )
                    class=move || button_style_classes(import_method.get() == ImportMethod::UrlTo)
                >
                    URL to
                    <br />
                    file or repo
                </button>
                <button
                    on:click=move |_| handle_import_method_button(
                        import_method,
                        ImportMethod::Upload,
                    )
                    class=move || button_style_classes(import_method.get() == ImportMethod::Upload)
                >
                    Upload
                    <br />
                    files or zip
                </button>
                <button
                    on:click=move |_| handle_import_method_button(
                        import_method,
                        ImportMethod::Paste,
                    )
                    class=move || button_style_classes(import_method.get() == ImportMethod::Paste)
                >
                    Paste in
                    <br />
                    text box
                </button>
            </div>

            {move || {
                (import_method.get() == ImportMethod::UrlTo)
                    .then_some(
                        view! {
                            <ImportFromUrlToSection
                                import_method
                                input_code_url
                                code_in_vstate
                                code_group
                                stage
                                placeholder="https://github.com/josehu07/codetective/tree/main"
                            />
                        },
                    )
            }}

            {move || {
                (import_method.get() == ImportMethod::Upload)
                    .then_some(
                        view! {
                            <ImportFromUploadSection
                                import_method
                                code_in_vstate
                                code_group
                                stage
                            />
                        },
                    )
            }}

            {move || {
                (import_method.get() == ImportMethod::Paste)
                    .then_some(
                        view! {
                            <ImportFromPasteSection
                                import_method
                                input_code_text
                                code_in_vstate
                                code_group
                                stage
                                placeholder="fn main() {\n    println!(\"Hello, detective!\");\n}\n"
                            />
                        },
                    )
            }}
        </div>
    }
}

#[component]
fn CodeRetrieveCollapsedView(
    import_method: RwSignal<ImportMethod>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
    stage: RwSignal<StepStage>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm">
            <StepHeaderCollapsed step=2 />

            <div class="text-center text-gray-800 text-lg">
                <span class="font-semibold">Code Source Imported:{NBSP}{NBSP}</span>
                <span class="text-xl font-mono">
                    {move || code_group.read().num_files()} {NBSP}File(s) {NBSP}
                    {move || {
                        code_group
                            .read()
                            .total_size()
                            .map_or(
                                "size unclear".to_string(),
                                |size| format!("~ {:.2}KB", (size as f32) / 1024.0),
                            )
                    }}
                </span>
            </div>

            {move || {
                (code_group.read().has_skipped())
                    .then_some(
                        view! {
                            <div class="text-orange-700 text-base font-mono mt-4 text-center animate-fade-in">
                                Some file(s) of size larger than {NBSP} {MAX_FILE_SIZE / 1024}KB
                                {NBSP}were skipped...
                            </div>
                        },
                    )
            }}
            {move || {
                (code_group.read().num_files() >= MAX_NUM_FILES)
                    .then_some(
                        view! {
                            <div class="text-orange-700 text-base font-mono mt-4 text-center animate-fade-in">
                                Number of files imported is currently capped at {NBSP}
                                {MAX_NUM_FILES} {NBSP}...
                            </div>
                        },
                    )
            }}

            {move || {
                (import_method.get() != ImportMethod::Null)
                    .then_some(
                        view! {
                            <button
                                on:click=move |_| handle_back_button(
                                    code_in_vstate,
                                    code_group,
                                    task_queue,
                                    num_finished,
                                    detection_cp,
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

/// The code retrieval step wrapped in one place.
#[component]
pub(crate) fn CodeRetrieve(
    import_method: RwSignal<ImportMethod>,
    input_code_url: RwSignal<String>,
    input_code_text: RwSignal<String>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    task_queue: RwSignal<TaskQueue>,
    num_finished: RwSignal<usize>,
    detection_cp: RwSignal<bool>,
    stage: RwSignal<StepStage>,
) -> impl IntoView {
    view! {
        {move || {
            (stage.get() == StepStage::ApiDone)
                .then_some(
                    view! {
                        <CodeRetrieveExpandedView
                            import_method
                            input_code_url
                            input_code_text
                            code_in_vstate
                            code_group
                            stage
                        />
                    },
                )
        }}

        {move || {
            (stage.get() > StepStage::ApiDone)
                .then_some(
                    view! {
                        <CodeRetrieveCollapsedView
                            import_method
                            code_in_vstate
                            code_group
                            task_queue
                            num_finished
                            detection_cp
                            stage
                        />
                    },
                )
        }}
    }
}
