#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn DialogFooter(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("flex flex-col-reverse sm:flex-row sm:justify-end sm:gap-x-2 {}", extra);

    view! {
        <div data-slot="dialog-footer" class=cls()>
            {children()}
        </div>
    }
}
