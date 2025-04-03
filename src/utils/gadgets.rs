//! Common reusable web page gadgets.

use std::cmp;

use leptos::prelude::*;

use crate::utils::NBSP;

/// An empty loading indicator that occupies the same space but is invisible.
#[component]
pub(crate) fn InvisibleIndicator() -> impl IntoView {
    view! { <div class="h-5 w-5 opacity-0"></div> }
}

/// A spinning loader circle.
#[component]
pub(crate) fn SpinningIndicator() -> impl IntoView {
    view! {
        <div class="animate-spin h-5 w-5 text-gray-500">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                ></circle>
                <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                ></path>
            </svg>
        </div>
    }
}

/// A row of blinking dots.
#[component]
pub(crate) fn BlinkDotsIndicator() -> impl IntoView {
    view! {
        <div class="flex justify-center">
            <div
                class="w-1.5 h-1.5 mx-0.5 bg-gray-500 rounded-full animate-pulse-fast"
                style="animation-delay: 0s"
            ></div>
            <div
                class="w-1.5 h-1.5 mx-0.5 bg-gray-500 rounded-full animate-pulse-fast"
                style="animation-delay: 0.2s"
            ></div>
            <div
                class="w-1.5 h-1.5 mx-0.5 bg-gray-500 rounded-full animate-pulse-fast"
                style="animation-delay: 0.4s"
            ></div>
        </div>
    }
}

/// A green check success indicator.
#[component]
pub(crate) fn SuccessIndicator() -> impl IntoView {
    let (is_bouncing, set_is_bouncing) = signal(true);
    Effect::new(move |_| {
        if is_bouncing.get() {
            set_timeout(
                move || {
                    set_is_bouncing.set(false);
                },
                std::time::Duration::from_millis(520), // animation duration + small buffer
            );
        }
    });

    view! {
        <div class=move || {
            if is_bouncing.get() {
                "h-5 w-5 text-green-700 animate-bounce-mid"
            } else {
                "h-5 w-5 text-green-700"
            }
        }>
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                <path
                    fill-rule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                    clip-rule="evenodd"
                />
            </svg>
        </div>
    }
}

/// A red cross failure indicator.
#[component]
pub(crate) fn FailureIndicator() -> impl IntoView {
    let (is_shaking, set_is_shaking) = signal(true);
    Effect::new(move |_| {
        if is_shaking.get() {
            set_timeout(
                move || {
                    set_is_shaking.set(false);
                },
                std::time::Duration::from_millis(520), // animation duration + small buffer
            );
        }
    });

    view! {
        <div class=move || {
            if is_shaking.get() {
                "h-5 w-5 text-red-700 animate-shake-fast"
            } else {
                "h-5 w-5 text-red-700"
            }
        }>
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                <path
                    fill-rule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                    clip-rule="evenodd"
                />
            </svg>
        </div>
    }
}

/// An information icon with custom hover title text.
#[component]
pub(crate) fn HoverInfoIcon(text: &'static str) -> impl IntoView {
    let show_popup = RwSignal::new(false);

    view! {
        <div class="relative">
            <div
                class="h-5 w-5 text-gray-500 hover:text-gray-700 cursor-help"
                on:mouseenter=move |_| show_popup.set(true)
                on:mouseleave=move |_| show_popup.set(false)
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                </svg>
            </div>

            <Show when=move || show_popup.get()>
                <div class="fixed bottom-0 left-2 z-40 p-2 bg-gray-800 text-white text-base text-left rounded shadow-lg w-max max-w-xl">
                    {text}
                </div>
            </Show>
        </div>
    }
}

/// A block that shows the percentage AI authorship result on it, and that
/// also shows an associated message when hovered over.
#[component]
pub(crate) fn HoverResultDiv(percent: Option<u8>, message: String) -> impl IntoView {
    let percent_s = match percent {
        Some(p) => format!("{}%", cmp::min(p, 100)),
        None => "-N/A-".to_string(),
    };
    let color_style = match percent {
        Some(p) => blended_color(p),
        None => "text-red-600 text-sm",
    };

    let show_popup = RwSignal::new(false);

    view! {
        <div class="relative">
            <div
                class={format!("w-16 h-6 leading-6 bg-gray-100 hover:bg-gray-300 rounded-md text-center align-middle text-base font-medium cursor-help animate-fade-in {}", color_style)}
                on:mouseenter=move |_| show_popup.set(true)
                on:mouseleave=move |_| show_popup.set(false)
            >
                {percent_s}
            </div>

            <Show when=move || show_popup.get()>
                <div class="fixed bottom-16 left-8 z-40 p-2 bg-gray-800 text-white text-base text-left rounded shadow-lg w-max max-w-lg">
                    {message.clone()}
                </div>
            </Show>
        </div>
    }
}

/// Calculates a blended color on a green-red spectrum given a ratio.
/// Currently uses a hardcoded, pre-calculated interpolation.
fn blended_color(red_percent: u8) -> &'static str {
    match red_percent {
        0..10 => "text-[#047608]",
        10..20 => "text-[#197804]",
        20..30 => "text-[#327904]",
        30..40 => "text-[#4d7b04]",
        40..50 => "text-[#687d04]",
        50..60 => "text-[#7e7a05]",
        60..70 => "text-[#806105]",
        70..80 => "text-[#824705]",
        80..90 => "text-[#832d05]",
        90.. => "text-[#851205]",
    }
}

/// A corner banner to GitHub.
#[component]
pub(crate) fn GitHubBanner() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            class="text-gray-500 hover:text-gray-600 fill-current"
            aria-hidden="true"
        >
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
        </svg>
    }
}

/// Step section top-left header: when expanded.
#[component]
pub(crate) fn StepHeaderExpanded(step: u8) -> impl IntoView {
    view! {
        <div class="absolute -top-3 -left-5 px-4 py-2 bg-gray-600 rounded-full flex items-center justify-center text-xl text-white font-semibold">
            Step{NBSP}{step}
        </div>
    }
}

/// Step section top-left header: when collapsed.
#[component]
pub(crate) fn StepHeaderCollapsed(step: u8) -> impl IntoView {
    view! {
        <div class="absolute -top-3 -left-5 px-4 py-2 bg-gray-400 rounded-full flex items-center justify-center text-base text-white font-semibold">
            Step{NBSP}{step}
        </div>
    }
}
