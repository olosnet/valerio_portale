#![allow(dead_code)]
use leptos::prelude::*;
use super::popover::use_popover;

#[component]
pub fn PopoverTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_popover();
    let extra = class.unwrap_or("");

    view! {
        <div
            data-slot="popover-trigger"
            on:click=move |_| ctx.open.update(|v| *v = !*v)
            class=extra
        >
            {children()}
        </div>
    }
}
