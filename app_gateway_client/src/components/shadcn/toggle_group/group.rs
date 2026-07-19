#![allow(dead_code)]
use leptos::prelude::use_context;
use std::sync::Arc;
use leptos::prelude::*;

#[derive(Clone)]
pub struct ToggleGroupContext {
    pub value: RwSignal<Option<String>>,
    pub on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

pub fn use_toggle_group() -> ToggleGroupContext {
    use_context::<ToggleGroupContext>().expect("ToggleGroupContext not provided")
}

#[component]
pub fn ToggleGroup(
    children: ChildrenFn,
    value: RwSignal<Option<String>>,
    #[prop(optional)] on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    provide_context(ToggleGroupContext { value, on_change });
    let extra = class.unwrap_or("");
    let cls = move || format!("inline-flex items-center justify-center gap-0 rounded-md bg-muted p-1 text-muted-foreground {}", extra);

    view! {
        <div data-slot="toggle-group" class=cls()>
            {children()}
        </div>
    }
}
