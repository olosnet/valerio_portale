#![allow(dead_code)]
use leptos::prelude::*;
use super::dropdown_menu::use_dropdown_menu;

#[component]
pub fn DropdownMenuTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_dropdown_menu();
    let extra = class.unwrap_or("");

    view! {
        <div
            data-slot="dropdown-menu-trigger"
            on:click=move |_| ctx.open.update(|v| *v = !*v)
            class=extra
        >
            {children()}
        </div>
    }
}
