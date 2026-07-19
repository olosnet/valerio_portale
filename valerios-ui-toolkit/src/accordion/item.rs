use leptos::prelude::*;

#[component]
pub fn AccordionItem(
    children: ChildrenFn,
    value: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    provide_context(value);

    view! {
        <div data-slot="accordion-item" class=format!("border-b {}", extra)>
            {children()}
        </div>
    }
}

pub fn use_accordion_item_value() -> &'static str {
    expect_context::<&'static str>()
}
