//! Step 2 section: import/retrieve code for analysis

use std::cmp;

use leptos::html::Div;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys::HtmlTextAreaElement;

use gloo_timers::future::TimeoutFuture;

use crate::file::{CodeGroup, MAX_FILE_SIZE, MAX_NUM_FILES};
use crate::utils::error::CodeImportError;
use crate::utils::gadgets::{
    FailureIndicator, HoverInfoIcon, InvisibleIndicator, SpinningIndicator, StepHeaderCollapsed,
    StepHeaderExpanded, SuccessIndicator,
};
use crate::utils::NBSP;
use crate::{Stage, ValidationState};

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
    stage: RwSignal<Stage>,
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
        match code_group_inner.add_remote(&code_url).await {
            Ok(()) => {
                code_in_vstate.set(ValidationState::Success);

                // small delay before proceeding to next stage
                TimeoutFuture::new(500).await;

                log::info!(
                    "Step 2 confirmed: imported {} file(s) from {}",
                    code_group_inner.num_files(),
                    current_import_method.name()
                );
                stage.set(Stage::CodeImported);
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
    stage: RwSignal<Stage>,
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
            "Step 2 validating: importing from {}...",
            current_import_method.name()
        );

        let mut code_group_inner = code_group.write();
        match code_group_inner.add_local(code_text).await {
            Ok(()) => {
                code_in_vstate.set(ValidationState::Success);

                // small delay before proceeding to next stage
                TimeoutFuture::new(500).await;

                log::info!(
                    "Step 2 confirmed: imported {} file(s) from {}",
                    code_group_inner.num_files(),
                    current_import_method.name()
                );
                stage.set(Stage::CodeImported);
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

fn handle_url_confirm_button(
    import_method: RwSignal<ImportMethod>,
    input_code_url: RwSignal<String>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<Stage>,
) {
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

fn handle_url_enter_key_down(
    import_method: RwSignal<ImportMethod>,
    input_code_url: RwSignal<String>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<Stage>,
) {
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

fn handle_text_confirm_button(
    import_method: RwSignal<ImportMethod>,
    input_code_text: RwSignal<String>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<Stage>,
) {
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

fn handle_back_button(
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<Stage>,
) {
    code_in_vstate.set(ValidationState::Idle);
    code_group.update(|cg| {
        cg.reset();
    });
    stage.set(Stage::ApiProvided);

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
    stage: RwSignal<Stage>,
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
                        if ev.key_code() != 0 && ev.key() == "Enter" {
                            handle_url_enter_key_down(
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
                        handle_url_confirm_button(
                            import_method,
                            input_code_url,
                            code_in_vstate,
                            code_group,
                            stage,
                        );
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
fn ImportFromUploadSection() -> impl IntoView {
    unimplemented!()
}

#[component]
fn ImportFromPasteSection(
    import_method: RwSignal<ImportMethod>,
    input_code_text: RwSignal<String>,
    code_in_vstate: RwSignal<ValidationState<CodeImportError>>,
    code_group: RwSignal<CodeGroup>,
    stage: RwSignal<Stage>,
    placeholder: &'static str,
) -> impl IntoView {
    view! {
        <div class="pt-6 pb-2 px-2 overflow-hidden animate-slide-down origin-top">
            <div class="flex flex-col items-center justify-center space-y-4">
                <div class="w-full flex items-center space-x-4">
                    <label for="code-textbox" class="text-base text-gray-900 whitespace-nowrap">
                        Paste code:
                    </label>
                    <div class="flex-1"></div>
                    <HoverInfoIcon text="Paste your source code directly into the text box. Size limited to 100KB, but code files should normally be much smaller than that." />
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
                            handle_text_confirm_button(
                                import_method,
                                input_code_text,
                                code_in_vstate,
                                code_group,
                                stage,
                            );
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
    stage: RwSignal<Stage>,
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
                    file or zip
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
    stage: RwSignal<Stage>,
) -> impl IntoView {
    view! {
        <div class="relative max-w-4xl w-full mt-12 px-8 py-6 bg-white/60 rounded-lg shadow-sm">
            <StepHeaderCollapsed step=2 />

            <div class="text-center text-gray-800 text-lg">
                <span class="font-semibold">Code Source Imported:{NBSP}{NBSP}</span>
                <span class="text-xl font-mono">
                    {move || code_group.read().num_files()} {NBSP}file(s) {NBSP}
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
                (code_group.read().skipped())
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
                                    stage,
                                )
                                class="absolute -bottom-3 -right-5 px-4 py-2 bg-gray-500 hover:bg-gray-600 rounded-md flex items-center justify-center text-white transition-colors"
                            >
                                Back
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
    stage: RwSignal<Stage>,
) -> impl IntoView {
    view! {
        {move || {
            (stage.get() == Stage::ApiProvided)
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
            (stage.get() > Stage::ApiProvided)
                .then_some(
                    view! {
                        <CodeRetrieveCollapsedView import_method code_in_vstate code_group stage />
                    },
                )
        }}
    }
}
