#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn SheetDescription(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("text-sm text-muted-foreground {}", extra);

    view! {
        <div data-slot="sheet-description" class=cls()>
            {children()}
        </div>
    }
}
