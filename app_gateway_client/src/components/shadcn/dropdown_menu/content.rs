#![allow(dead_code)]
use leptos::prelude::*;
use super::dropdown_menu::use_dropdown_menu;

#[component]
pub fn DropdownMenuContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = "bottom")] side: &'static str,
) -> impl IntoView {
    let ctx = use_dropdown_menu();
    let extra = class.unwrap_or("");

    let side_class = match side {
        "top" => "bottom-full left-0 mb-2",
        "bottom" => "top-full left-0 mt-1",
        _ => "top-full left-0 mt-1",
    };

    let cls = move || format!(
        "absolute z-50 min-w-[8rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md animate-fade-in {} {}",
        side_class, extra,
    );

    let handle_backdrop = move |_| ctx.open.set(false);
    let handle_content_click = move |ev: leptos::ev::MouseEvent| ev.stop_propagation();
    let handle_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Escape" { ctx.open.set(false); }
    };

    move || {
        if ctx.open.get() {
            view! {
                <div on:click=handle_backdrop class="fixed inset-0 z-40" />
                <div
                    data-slot="dropdown-menu-content"
                    on:click=handle_content_click
                    on:keydown=handle_keydown
                    role="menu"
                    class=cls()
                >
                    {children()}
                </div>
            }.into_any()
        } else { ().into_any() }
    }
}
