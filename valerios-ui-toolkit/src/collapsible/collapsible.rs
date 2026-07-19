use leptos::prelude::*;

#[derive(Clone)]
pub struct CollapsibleContext {
    pub open: RwSignal<bool>,
}

pub fn use_collapsible() -> CollapsibleContext {
    expect_context::<CollapsibleContext>()
}

#[component]
pub fn Collapsible(
    children: ChildrenFn,
    open: RwSignal<bool>,
) -> impl IntoView {
    provide_context(CollapsibleContext { open });

    view! {
        <div data-slot="collapsible" class="w-full">
            {children()}
        </div>
    }
}
