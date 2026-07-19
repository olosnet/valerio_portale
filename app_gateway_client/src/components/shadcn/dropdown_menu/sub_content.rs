#![allow(dead_code)]
use leptos::prelude::*;
use super::sub::use_menu_sub;

#[component]
pub fn DropdownMenuSubContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_menu_sub();
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "absolute left-full top-0 z-50 ml-1 min-w-[8rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md animate-fade-in {}",
        extra,
    );

    let handle_backdrop = move |_| ctx.open.set(false);
    let handle_content_click = move |ev: leptos::ev::MouseEvent| ev.stop_propagation();

    move || {
        if ctx.open.get() {
            view! {
                <div on:click=handle_backdrop class="fixed inset-0 z-40" />
                <div
                    data-slot="dropdown-menu-sub-content"
                    on:click=handle_content_click
                    class=cls()
                >
                    {children()}
                </div>
            }.into_any()
        } else { ().into_any() }
    }
}
