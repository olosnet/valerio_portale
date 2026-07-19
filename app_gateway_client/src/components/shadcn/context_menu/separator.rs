#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn ContextMenuSeparator(
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <div role="separator" data-slot="context-menu-separator" class=format!("-mx-1 my-1 h-px bg-muted {}", extra) />
    }
}
