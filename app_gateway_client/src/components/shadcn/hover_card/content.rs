#![allow(dead_code)]
use leptos::prelude::*;
use super::hover_card::use_hover_card;

#[component]
pub fn HoverCardContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_hover_card();
    let extra = class.unwrap_or("");

    move || {
        if ctx.open.get() {
            view! {
                <div data-slot="hover-card-content" role="tooltip"
                    class=format!("absolute z-50 w-64 rounded-md border bg-popover p-4 text-popover-foreground shadow-md animate-fade-in top-full left-1/2 -translate-x-1/2 mt-2 {}", extra)>
                    {children()}
                </div>
            }.into_any()
        } else { ().into_any() }
    }
}
