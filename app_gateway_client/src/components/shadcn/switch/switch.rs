#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;

#[component]
pub fn Switch(
    checked: RwSignal<bool>,
    #[prop(optional)] on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] id: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let base = "peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input";
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
            role="switch"
            disabled=disabled
            aria-checked=move || if checked.get() { "true" } else { "false" }
            data-state=move || if checked.get() { "checked" } else { "unchecked" }
            on:click=handle_click
            class=cls()
        >
            <span
                data-state=move || if checked.get() { "checked" } else { "unchecked" }
                class="pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0"
            />
        </button>
    }
}
