#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn BreadcrumbItem(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <li data-slot="breadcrumb-item" class=format!("inline-flex items-center gap-1.5 {}", extra)>
            {children()}
        </li>
    }
}
