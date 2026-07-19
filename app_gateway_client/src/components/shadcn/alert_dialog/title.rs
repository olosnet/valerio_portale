#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn AlertDialogTitle(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("text-lg font-semibold {}", extra);

    view! {
        <div data-slot="alert-dialog-title" class=cls()>
            {children()}
        </div>
    }
}
