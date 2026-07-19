use leptos::prelude::*;

#[derive(Clone)]
pub struct TabsContext {
    pub value: RwSignal<String>,
}

impl TabsContext {
    pub fn new(initial: &str) -> Self {
        Self {
            value: RwSignal::new(initial.to_string()),
        }
    }
}

pub fn use_tabs() -> TabsContext {
    expect_context::<TabsContext>()
}

#[component]
pub fn Tabs(
    children: ChildrenFn,
    value: RwSignal<String>,
) -> impl IntoView {
    provide_context(TabsContext { value });

    view! {
        <div data-slot="tabs">
            {children()}
        </div>
    }
}
