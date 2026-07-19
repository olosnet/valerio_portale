#![allow(dead_code)]
use leptos::prelude::*;
use super::{accordion::use_accordion, item::use_accordion_item_value};

#[component]
pub fn AccordionContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_accordion();
    let value = use_accordion_item_value();
    let extra = class.unwrap_or("");
    let base = "overflow-hidden text-sm data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down";

    move || {
        if ctx.open_item.get().as_deref() == Some(value) {
            view! {
                <div
                    data-state="open"
                    data-slot="accordion-content"
                    class=format!("{} pb-4 pt-0 {}", base, extra)
                >
                    {children()}
                </div>
            }.into_any()
        } else {
            ().into_any()
        }
    }
}
