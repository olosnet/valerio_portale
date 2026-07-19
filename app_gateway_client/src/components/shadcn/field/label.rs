#![allow(dead_code)]
use leptos::prelude::*;
use super::field::use_field;

#[component]
pub fn FieldLabel(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_field();
    let extra = class.unwrap_or("");

    view! {
        <label data-slot="field-label" for=ctx.id class=format!("text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 {}", extra)>
            {children()}
        </label>
    }
}
