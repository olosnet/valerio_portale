use leptos::prelude::*;
use super::tooltip::use_tooltip;

#[component]
pub fn TooltipTrigger(
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = use_tooltip();

    view! {
        <div
            data-slot="tooltip-trigger"
            on:mouseenter=move |_| ctx.open.set(true)
            on:mouseleave=move |_| ctx.open.set(false)
        >
            {children()}
        </div>
    }
}
