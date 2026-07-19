#![allow(dead_code)]
use leptos::prelude::*;
use super::group::use_toggle_group;

#[component]
pub fn ToggleGroupItem(
    children: ChildrenFn,
    value: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_toggle_group();
    let extra = class.unwrap_or("");
    let cls = move || {
        let active = ctx.value.get().as_deref() == Some(value);
        format!(
            "inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=on]:bg-background data-[state=on]:text-foreground data-[state=on]:shadow-sm {} {}",
            extra,
            if active { "bg-background text-foreground shadow-sm" } else { "hover:bg-accent hover:text-accent-foreground" },
        )
    };

    let handle_click = move |_| {
        let current = ctx.value.get();
        if current.as_deref() == Some(value) {
            ctx.value.set(None);
        } else {
            ctx.value.set(Some(value.to_string()));
            if let Some(ref cb) = ctx.on_change {
                cb(value.to_string());
            }
        }
    };

    view! {
        <button
            type="button"
            data-slot="toggle-group-item"
            data-state=move || if ctx.value.get().as_deref() == Some(value) { "on" } else { "off" }
            on:click=handle_click
            class=cls()
        >
            {children()}
        </button>
    }
}
