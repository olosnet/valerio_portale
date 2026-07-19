#![allow(dead_code)]
use leptos::prelude::*;

#[derive(Clone)]
pub struct MenubarMenuContext {
    pub open: RwSignal<bool>,
}

impl MenubarMenuContext {
    pub fn new() -> Self { Self { open: RwSignal::new(false) } }
}

pub fn use_menubar_menu() -> MenubarMenuContext {
    expect_context::<MenubarMenuContext>()
}

#[component]
pub fn MenubarMenu(
    children: ChildrenFn,
) -> impl IntoView {
    provide_context(MenubarMenuContext::new());

    view! {
        <div data-slot="menubar-menu" class="relative">
            {children()}
        </div>
    }
}
