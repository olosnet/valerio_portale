#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;

#[derive(Clone)]
pub struct RadioGroupContext {
    pub value: RwSignal<String>,
    pub on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

pub fn use_radio_group() -> RadioGroupContext {
    expect_context::<RadioGroupContext>()
}

#[component]
pub fn RadioGroup(
    children: ChildrenFn,
    value: RwSignal<String>,
    #[prop(optional)] on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    provide_context(RadioGroupContext { value, on_change });
    let extra = class.unwrap_or("");
    let cls = move || format!("grid gap-2 {}", extra);

    view! {
        <div role="radiogroup" data-slot="radio-group" class=cls()>
            {children()}
        </div>
    }
}
