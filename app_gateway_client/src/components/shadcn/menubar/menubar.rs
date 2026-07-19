#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn Menubar(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("flex h-10 items-center gap-1 rounded-md border bg-background p-1 shadow-sm {}", extra);

    view! {
        <div data-slot="menubar" class=cls()>
            {children()}
        </div>
    }
}
