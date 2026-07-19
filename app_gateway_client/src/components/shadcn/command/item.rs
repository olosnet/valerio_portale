#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;

#[component]
pub fn CommandItem(
    children: ChildrenFn,
    #[prop(optional)] on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = "")] keywords: &'static str,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "relative flex cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-sm ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&>svg]:size-4 [&>svg]:shrink-0 {}",
        extra,
    );

    let handle = move |_| {
        if !disabled {
            if let Some(ref cb) = on_click { cb(); }
        }
    };

    view! {
        <div
            role="option"
            data-slot="command-item"
            data-disabled=if disabled { "true" } else { "false" }
            on:click=handle
            class=cls()
        >
            {children()}
        </div>
    }
}
