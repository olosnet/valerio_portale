use leptos::prelude::*;

#[derive(Clone)]
pub struct HoverCardContext {
    pub open: RwSignal<bool>,
}

impl HoverCardContext {
    pub fn new() -> Self {
        Self { open: RwSignal::new(false) }
    }
}

pub fn use_hover_card() -> HoverCardContext {
    expect_context::<HoverCardContext>()
}

#[component]
pub fn HoverCard(
    children: ChildrenFn,
    #[prop(default = 300)] delay_ms: u32,
) -> impl IntoView {
    provide_context(HoverCardContext::new());

    view! {
        <div data-slot="hover-card" class="relative inline-flex">
            {children()}
        </div>
    }
}
