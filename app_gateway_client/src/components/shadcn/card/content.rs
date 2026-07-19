#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn CardContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("p-6 pt-0 {}", extra);

    view! {
        <div data-slot="card-content" class=cls()>
            {children()}
        </div>
    }
}
