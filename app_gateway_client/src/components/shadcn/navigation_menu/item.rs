#![allow(dead_code)]
use leptos::prelude::*;

#[derive(Clone)]
pub struct NavMenuItemContext {
    pub open: RwSignal<bool>,
}

impl NavMenuItemContext {
    pub fn new() -> Self { Self { open: RwSignal::new(false) } }
}

pub fn use_nav_menu_item() -> NavMenuItemContext {
    expect_context::<NavMenuItemContext>()
}

#[component]
pub fn NavigationMenuItem(
    children: ChildrenFn,
) -> impl IntoView {
    provide_context(NavMenuItemContext::new());

    view! {
        <li data-slot="navigation-menu-item" class="relative">
            {children()}
        </li>
    }
}
