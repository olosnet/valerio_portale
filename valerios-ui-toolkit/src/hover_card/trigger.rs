use leptos::prelude::*;
use super::hover_card::use_hover_card;

#[component]
pub fn HoverCardTrigger(
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = use_hover_card();

    view! {
        <div data-slot="hover-card-trigger"
            on:mouseenter=move |_| ctx.open.set(true)
            on:mouseleave=move |_| ctx.open.set(false)>
            {children()}
        </div>
    }
}
