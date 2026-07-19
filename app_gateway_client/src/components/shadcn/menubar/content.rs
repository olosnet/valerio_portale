#![allow(dead_code)]
use leptos::prelude::*;
use super::menu::use_menubar_menu;

#[component]
pub fn MenubarContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_menubar_menu();
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "absolute z-50 left-0 top-full mt-1 min-w-[12rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md animate-fade-in {}",
        extra,
    );

    let backdrop = move |_| ctx.open.set(false);
    let stop = move |ev: leptos::ev::MouseEvent| ev.stop_propagation();
    let escape = move |ev: leptos::ev::KeyboardEvent| { if ev.key() == "Escape" { ctx.open.set(false); } };

    move || {
        if ctx.open.get() {
            view! {
                <div on:click=backdrop class="fixed inset-0 z-40" />
                <div data-slot="menubar-content" on:click=stop on:keydown=escape class=cls() role="menu">
                    {children()}
                </div>
            }.into_any()
        } else { ().into_any() }
    }
}
