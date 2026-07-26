use leptos::prelude::*;

#[derive(Clone)]
pub struct TooltipContext {
    pub open: RwSignal<bool>,
}

impl TooltipContext {
    pub fn new() -> Self {
        Self {
            open: RwSignal::new(false),
        }
    }
}

pub fn use_tooltip() -> TooltipContext {
    expect_context::<TooltipContext>()
}

#[component]
pub fn Tooltip(
    children: ChildrenFn,
    // #[prop(default = 300)] delay_ms: u32,
) -> impl IntoView {
    provide_context(TooltipContext::new());

    view! {
        <div class="relative inline-flex" data-slot="tooltip">
            {children()}
        </div>
    }
}
