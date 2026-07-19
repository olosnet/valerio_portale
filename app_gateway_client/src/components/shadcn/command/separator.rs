#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn CommandSeparator(
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <div role="separator" data-slot="command-separator" class=format!("-mx-1 h-px bg-border {}", extra) />
    }
}
