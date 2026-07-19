#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn FieldDescription(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <p data-slot="field-description" class=format!("text-sm text-muted-foreground {}", extra)>
            {children()}
        </p>
    }
}
