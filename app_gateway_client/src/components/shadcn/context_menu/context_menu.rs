#![allow(dead_code)]
use leptos::prelude::*;

#[derive(Clone)]
pub struct ContextMenuContext {
    pub open: RwSignal<bool>,
    pub x: RwSignal<f64>,
    pub y: RwSignal<f64>,
}

impl ContextMenuContext {
    pub fn new() -> Self {
        Self {
            open: RwSignal::new(false),
            x: RwSignal::new(0.0),
            y: RwSignal::new(0.0),
        }
    }
}

pub fn use_context_menu() -> ContextMenuContext {
    expect_context::<ContextMenuContext>()
}

#[component]
pub fn ContextMenu(
    children: ChildrenFn,
) -> impl IntoView {
    provide_context(ContextMenuContext::new());

    view! {
        <div data-slot="context-menu" class="relative">
            {children()}
        </div>
    }
}
