#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn TableRow(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] on_click: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted {}", extra);
    let handle = move |_| { if let Some(ref cb) = on_click { cb(); } };

    view! {
        <tr data-slot="table-row" class=cls() on:click=handle>
            {children()}
        </tr>
    }
}
