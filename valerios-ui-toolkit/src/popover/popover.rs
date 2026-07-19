use leptos::prelude::*;

#[derive(Clone)]
pub struct PopoverContext {
    pub open: RwSignal<bool>,
}

impl PopoverContext {
    pub fn new(default_open: bool) -> Self {
        Self { open: RwSignal::new(default_open) }
    }
}

pub fn use_popover() -> PopoverContext {
    expect_context::<PopoverContext>()
}

#[component]
pub fn Popover(
    children: ChildrenFn,
    open: RwSignal<bool>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    provide_context(PopoverContext { open });
    let extra = class.unwrap_or("");

    view! {
        <div data-slot="popover" class=format!("relative inline-block {}", extra)>
            {children()}
        </div>
    }
}
