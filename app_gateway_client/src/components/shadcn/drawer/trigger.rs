#![allow(dead_code)]
use leptos::prelude::*;
use crate::components::shadcn::shared::use_overlay;

#[component]
pub fn DrawerTrigger(
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = use_overlay();
    view! {
        <div data-slot="drawer-trigger" on:click=move |_| ctx.open.set(true)>
            {children()}
        </div>
    }
}
