#![allow(dead_code)]
use leptos::prelude::*;
use crate::components::shadcn::shared::OverlayClose;

#[component]
pub fn DrawerClose(
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <OverlayClose>
            {children()}
        </OverlayClose>
    }
}
