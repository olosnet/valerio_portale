#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn BreadcrumbPage(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <span role="link" aria-disabled="true" aria-current="page" data-slot="breadcrumb-page" class=format!("font-normal text-foreground {}", extra)>
            {children()}
        </span>
    }
}
