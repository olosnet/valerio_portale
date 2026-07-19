use leptos::prelude::*;

#[derive(Clone)]
pub struct MenuSubContext {
    pub open: RwSignal<bool>,
}

impl MenuSubContext {
    pub fn new() -> Self {
        Self { open: RwSignal::new(false) }
    }
}

pub fn use_menu_sub() -> MenuSubContext {
    expect_context::<MenuSubContext>()
}

#[component]
pub fn DropdownMenuSub(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    provide_context(MenuSubContext::new());
    let extra = class.unwrap_or("");

    view! {
        <div data-slot="dropdown-menu-sub" class=format!("relative {}", extra)>
            {children()}
        </div>
    }
}
