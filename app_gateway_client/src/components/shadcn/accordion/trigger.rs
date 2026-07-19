#![allow(dead_code)]
use leptos::prelude::*;
use super::{accordion::use_accordion, item::use_accordion_item_value};

#[component]
pub fn AccordionTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_accordion();
    let value = use_accordion_item_value();
    let extra = class.unwrap_or("");
    let cls = move || {
        let is_open = ctx.open_item.get().as_deref() == Some(value);
        format!(
            "flex flex-1 items-center justify-between py-4 text-sm font-medium ring-offset-background transition-all hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&[data-state=open]>svg]:rotate-180 {}",
            extra,
        )
    };

    let handle_click = move |_| {
        let current = ctx.open_item.get();
        if current.as_deref() == Some(value) {
            ctx.open_item.set(None);
        } else {
            ctx.open_item.set(Some(value.to_string()));
        }
    };

    view! {
        <h3 data-slot="accordion-trigger" class="flex">
            <button
                type="button"
                data-state=move || if ctx.open_item.get().as_deref() == Some(value) { "open" } else { "closed" }
                on:click=handle_click
                class=cls()
            >
                {children()}
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4 shrink-0 text-muted-foreground transition-transform duration-200">
                    <path d="m6 9 6 6 6-6"/>
                </svg>
            </button>
        </h3>
    }
}
