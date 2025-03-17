//! Main entrance to the codetective web app.

use leptos::prelude::*;

use leptos_meta::{provide_meta_context, Link, Stylesheet, Title};

use leptos_router::components::{Route, Router, Routes};
use leptos_router::StaticSegment;

mod api_selection;
use api_selection::{ApiProvider, ApiSelection};

mod code_analysis;
use code_analysis::CodeAnalysis;

pub(crate) const NBSP: &str = "\u{00A0}"; // space
pub(crate) const NBHY: &str = "\u{2011}"; // hyphen

/// Stage enum that controls where are we in the workflow.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Stage {
    Initial,
    ApiProvided,
    CodeAnalyzed,
}

/// Currently, the app only has one route, which is the home page.
#[component]
fn Home() -> impl IntoView {
    let (stage, set_stage) = signal(Stage::Initial);
    let (api_provider, set_api_provider) = signal(ApiProvider::Null);
    let (api_key, set_api_key) = signal(String::new());

    view! {
        <Title text="Codetective"/>
        <main>
            <div class="bg-gradient-to-tl from-gray-300 to-gray-200 text-black font-sans flex flex-col max-w-full min-h-screen">
                <div class="flex flex-col items-center pt-16">
                    <div class="flex flex-col items-center">
                        <div class="flex items-end justify-center">
                            <h1 class="text-5xl font-bold text-gray-600">Co</h1>
                            <h1 class="text-5xl font-bold text-gray-900">de</h1>
                            <h1 class="text-5xl font-bold text-gray-600">tective</h1>
                            <img src="/codetective.png" alt="Codetective Logo" class="ml-4 h-16" />
                        </div>
                        <h2 class="text-2xl font-semibold text-gray-600 mt-4">
                            Code AI Authorship Detection in 3 Clicks
                        </h2>
                    </div>

                    <ApiSelection
                        api_provider
                        set_api_provider
                        set_api_key
                        stage
                        set_stage
                    />

                    <CodeAnalysis api_provider api_key stage set_stage/>
                </div>

                <footer class="mt-auto py-6 text-center">
                    <p class="text-xs text-gray-500">
                        Made with{NBSP}
                        <a href="https://leptos.dev" target="_blank" rel="noopener noreferrer" class="text-blue-700 hover:underline">Rust Leptos</a>{NBSP}+{NBSP}
                        <a href="https://tailwindcss.com" target="_blank" rel="noopener noreferrer" class="text-blue-700 hover:underline">Tailwind CSS</a>{NBSP}+{NBSP}
                        <a href="https://trunkrs.dev" target="_blank" rel="noopener noreferrer" class="text-blue-700 hover:underline">Trunk WASM</a>.{NBSP}
                        Originally authored by{NBSP}
                        <a href="https://josehu.com" target="_blank" rel="noopener noreferrer" class="text-blue-700 hover:underline">Guanzhou Hu</a>.
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

    view! {
        <Stylesheet id="leptos" href="/style/output.css"/>
        <Link rel="icon" type_="image/png" href="/icon/favicon-96x96.png"/>
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=StaticSegment("") view=Home/>
            </Routes>
        </Router>
    }
}

fn main() {
    _ = console_log::init_with_level(log::Level::Info);
    console_error_panic_hook::set_once();

    mount_to_body(App)
}
