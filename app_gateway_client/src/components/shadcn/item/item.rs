#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn Item(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] on_click: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("flex items-center gap-2 rounded-md p-2 hover:bg-accent cursor-pointer {}", extra);
    let handle = move |_| { if let Some(ref cb) = on_click { cb(); } };

    view! {
        <div data-slot="item" class=cls() on:click=handle>
            {children()}
        </div>
    }
}
