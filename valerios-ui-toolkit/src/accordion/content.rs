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
    let base = "grid transition-[grid-template-rows] duration-300";

    move || {
        let is_open = ctx.open_item.get().as_deref() == Some(value);
        view! {
            <div
                data-state=if is_open { "open" } else { "closed" }
                data-slot="accordion-content"
                class=format!("{} grid-rows-[0fr] data-[state=open]:grid-rows-[1fr] {} {}", base, extra, if is_open { "pb-4 pt-0" } else { "" })
            >
                <div class="overflow-hidden">
                    {children()}
                </div>
            </div>
        }.into_any()
    }
}
