#![allow(dead_code)]
use leptos::prelude::*;

#[derive(Clone)]
pub struct DropdownMenuContext {
    pub open: RwSignal<bool>,
}

impl DropdownMenuContext {
    pub fn new(default_open: bool) -> Self {
        Self { open: RwSignal::new(default_open) }
    }
}

pub fn use_dropdown_menu() -> DropdownMenuContext {
    expect_context::<DropdownMenuContext>()
}

#[component]
pub fn DropdownMenu(
    children: ChildrenFn,
    open: RwSignal<bool>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    provide_context(DropdownMenuContext { open });
    let extra = class.unwrap_or("");

    view! {
        <div data-slot="dropdown-menu" class=format!("relative inline-block {}", extra)>
            {children()}
        </div>
    }
}
