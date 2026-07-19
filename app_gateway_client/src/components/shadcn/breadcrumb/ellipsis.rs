#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn BreadcrumbEllipsis(
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <span role="presentation" aria-hidden="true" data-slot="breadcrumb-ellipsis" class=format!("flex size-9 items-center justify-center {}", extra)>
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4">
                <circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>
            </svg>
        </span>
    }
}
