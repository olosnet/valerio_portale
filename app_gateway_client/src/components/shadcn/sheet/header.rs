#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn SheetHeader(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("flex flex-col gap-y-1.5 text-center sm:text-left {}", extra);

    view! {
        <div data-slot="sheet-header" class=cls()>
            {children()}
        </div>
    }
}
