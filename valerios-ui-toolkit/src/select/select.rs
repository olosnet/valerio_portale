use std::sync::Arc;
use leptos::prelude::*;

#[derive(Clone)]
pub struct SelectContext {
    pub open: RwSignal<bool>,
    pub value: RwSignal<String>,
    pub on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl SelectContext {
    pub fn new(initial: &str) -> Self {
        Self {
            open: RwSignal::new(false),
            value: RwSignal::new(initial.to_string()),
            on_change: None,
        }
    }
}

pub fn use_select() -> SelectContext {
    expect_context::<SelectContext>()
}

#[component]
pub fn Select(
    children: ChildrenFn,
    value: RwSignal<String>,
    #[prop(optional)] on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    provide_context(SelectContext {
        open: RwSignal::new(false),
        value,
        on_change,
    });
    let extra = class.unwrap_or("");

    view! {
        <div data-slot="select" class=format!("relative {}", extra)>
            {children()}
        </div>
    }
}
