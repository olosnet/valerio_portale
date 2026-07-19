#![allow(dead_code)]
use leptos::prelude::*;
use super::menu::use_menubar_menu;

#[component]
pub fn MenubarTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_menubar_menu();
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 data-[state=open]:bg-accent data-[state=open]:text-accent-foreground {}",
        extra,
    );

    view! {
        <button type="button" data-slot="menubar-trigger"
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            on:click=move |_| ctx.open.update(|v| *v = !*v)
            class=cls()>
            {children()}
        </button>
    }
}
