#![allow(dead_code)]
use leptos::prelude::*;
use super::collapsible::use_collapsible;

#[component]
pub fn CollapsibleTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_collapsible();
    let extra = class.unwrap_or("");

    view! {
        <button type="button" data-slot="collapsible-trigger"
            on:click=move |_| ctx.open.update(|v| *v = !*v)
            class=format!("flex w-full items-center justify-between {}", extra)>
            {children()}
        </button>
    }
}
