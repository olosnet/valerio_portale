#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;

#[component]
pub fn Checkbox(
    checked: RwSignal<bool>,
    #[prop(optional)] on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] id: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let base = "peer h-4 w-4 shrink-0 rounded-sm border border-primary ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground";
    let cls = move || format!("{} {}", base, extra);

    let handle_click = move |_| {
        if disabled { return; }
        let new_val = !checked.get();
        checked.set(new_val);
        if let Some(ref cb) = on_change {
            cb(new_val);
        }
    };

    view! {
        <button
            id=id
            type="button"
            role="checkbox"
            disabled=disabled
            aria-checked=move || if checked.get() { "true" } else { "false" }
            data-state=move || if checked.get() { "checked" } else { "unchecked" }
            on:click=handle_click
            class=cls()
        >
            {move || if checked.get() {
                view! {
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="size-3.5">
                        <path d="M20 6 9 17l-5-5"/>
                    </svg>
                }.into_any()
            } else {
                ().into_any()
            }}
        </button>
    }
}
