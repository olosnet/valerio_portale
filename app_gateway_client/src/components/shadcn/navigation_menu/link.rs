#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;

#[component]
pub fn NavigationMenuLink(
    children: ChildrenFn,
    #[prop(optional)] on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    #[prop(default = false)] active: bool,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 {} {}",
        extra,
        if active { "bg-accent text-accent-foreground" } else { "" },
    );
    let handle = move |_| { if let Some(ref cb) = on_click { cb(); } };

    view! {
        <button type="button" data-slot="navigation-menu-link" on:click=handle class=cls()>
            {children()}
        </button>
    }
}
