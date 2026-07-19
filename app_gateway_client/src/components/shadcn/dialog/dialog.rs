#![allow(dead_code)]
use leptos::prelude::*;
use crate::components::shadcn::shared::OverlayProvider;

#[component]
pub fn Dialog(
    children: ChildrenFn,
    open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <OverlayProvider open=open>
            {children()}
        </OverlayProvider>
    }
}
