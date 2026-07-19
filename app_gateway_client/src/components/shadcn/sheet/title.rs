#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn SheetTitle(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("text-lg font-semibold text-foreground {}", extra);

    view! {
        <div data-slot="sheet-title" class=cls()>
            {children()}
        </div>
    }
}
