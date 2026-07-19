#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn Card(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("rounded-lg border bg-card text-card-foreground shadow-sm {}", extra);

    view! {
        <div data-slot="card" class=cls()>
            {children()}
        </div>
    }
}
