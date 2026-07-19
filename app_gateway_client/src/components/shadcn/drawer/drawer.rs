#![allow(dead_code)]
use leptos::prelude::*;
use crate::components::shadcn::shared::OverlayProvider;

#[component]
pub fn Drawer(
    children: ChildrenFn,
    open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <OverlayProvider open=open>
            <div data-slot="drawer" class="relative">
                {children()}
            </div>
        </OverlayProvider>
    }
}
