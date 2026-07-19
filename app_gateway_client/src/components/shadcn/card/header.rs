#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn CardHeader(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("flex flex-col gap-y-1.5 p-6 {}", extra);

    view! {
        <div data-slot="card-header" class=cls()>
            {children()}
        </div>
    }
}
