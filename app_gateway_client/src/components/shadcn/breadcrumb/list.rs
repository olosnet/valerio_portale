#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn BreadcrumbList(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("flex flex-wrap items-center gap-1.5 break-words text-sm text-muted-foreground sm:gap-2.5 {}", extra);

    view! {
        <ol data-slot="breadcrumb-list" class=cls()>
            {children()}
        </ol>
    }
}
