use std::sync::Arc;
use leptos::prelude::*;

#[component]
pub fn BreadcrumbLink(
    children: ChildrenFn,
    #[prop(optional)] on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("transition-colors hover:text-foreground {}", extra);
    let handle = move |_| { if let Some(ref cb) = on_click { cb(); } };

    view! {
        <button type="button" on:click=handle data-slot="breadcrumb-link" class=cls()>
            {children()}
        </button>
    }
}
