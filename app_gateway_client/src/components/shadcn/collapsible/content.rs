#![allow(dead_code)]
use leptos::prelude::*;
use super::collapsible::use_collapsible;

#[component]
pub fn CollapsibleContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_collapsible();
    let extra = class.unwrap_or("");

    move || {
        if ctx.open.get() {
            view! {
                <div data-slot="collapsible-content" class=format!("overflow-hidden {}", extra)>
                    {children()}
                </div>
            }.into_any()
        } else { ().into_any() }
    }
}
