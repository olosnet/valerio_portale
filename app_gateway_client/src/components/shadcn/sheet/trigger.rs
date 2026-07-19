#![allow(dead_code)]
use leptos::prelude::*;
use crate::components::shadcn::shared::use_overlay;

#[component]
pub fn SheetTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_overlay();
    let extra = class.unwrap_or("");

    view! {
        <div
            data-slot="sheet-trigger"
            on:click=move |_| ctx.open.set(true)
            class=extra
        >
            {children()}
        </div>
    }
}
