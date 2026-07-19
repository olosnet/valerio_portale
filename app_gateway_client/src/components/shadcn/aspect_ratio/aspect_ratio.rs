#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn AspectRatio(
    children: ChildrenFn,
    #[prop(default = 1.0)] ratio: f64,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <div
            data-slot="aspect-ratio"
            class=format!("relative w-full {}", extra)
            style=format!("padding-bottom: {}%", 100.0 / ratio)
        >
            <div class="absolute inset-0">
                {children()}
            </div>
        </div>
    }
}
