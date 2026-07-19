#![allow(dead_code)]
use leptos::prelude::*;
use super::context_menu::use_context_menu;

#[component]
pub fn ContextMenuTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_context_menu();
    let extra = class.unwrap_or("");

    let handle_context = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        ctx.x.set(ev.client_x() as f64);
        ctx.y.set(ev.client_y() as f64);
        ctx.open.set(true);
    };

    view! {
        <div
            data-slot="context-menu-trigger"
            on:contextmenu=handle_context
            class=extra
        >
            {children()}
        </div>
    }
}
