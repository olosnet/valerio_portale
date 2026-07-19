#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn Avatar(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full {}", extra);

    view! {
        <span data-slot="avatar" class=cls()>
            {children()}
        </span>
    }
}
