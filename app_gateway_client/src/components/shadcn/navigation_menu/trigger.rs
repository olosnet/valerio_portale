#![allow(dead_code)]
use leptos::prelude::*;
use super::item::use_nav_menu_item;

#[component]
pub fn NavigationMenuTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_nav_menu_item();
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 group-data-[state=open]:bg-accent group-data-[state=open]:text-accent-foreground {}",
        extra,
    );

    view! {
        <button type="button" data-slot="navigation-menu-trigger"
            on:click=move |_| ctx.open.update(|v| *v = !*v)
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            class=cls()>
            {children()}
        </button>
    }
}
