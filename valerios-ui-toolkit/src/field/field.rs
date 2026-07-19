use leptos::prelude::*;

#[derive(Clone)]
pub struct FieldContext {
    pub id: &'static str,
}

impl FieldContext {
    pub fn new(id: &'static str) -> Self { Self { id } }
}

pub fn use_field() -> FieldContext {
    expect_context::<FieldContext>()
}

#[component]
pub fn Field(
    children: ChildrenFn,
    id: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    provide_context(FieldContext { id });
    let extra = class.unwrap_or("");

    view! {
        <div data-slot="field" class=format!("grid gap-2 {}", extra)>
            {children()}
        </div>
    }
}
