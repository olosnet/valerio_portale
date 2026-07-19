use leptos::prelude::*;

#[derive(Clone)]
pub struct AccordionContext {
    pub open_item: RwSignal<Option<String>>,
}

impl AccordionContext {
    pub fn new() -> Self {
        Self {
            open_item: RwSignal::new(None),
        }
    }
}

pub fn use_accordion() -> AccordionContext {
    expect_context::<AccordionContext>()
}

#[component]
pub fn Accordion(
    children: ChildrenFn,
    #[prop(default = "")] default_value: &'static str,
) -> impl IntoView {
    let ctx = if default_value.is_empty() {
        AccordionContext::new()
    } else {
        AccordionContext {
            open_item: RwSignal::new(Some(default_value.to_string())),
        }
    };

    provide_context(ctx.clone());

    view! {
        <div data-slot="accordion" class="w-full">
            {children()}
        </div>
    }
}
