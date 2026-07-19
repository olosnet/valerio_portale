use std::sync::Arc;

use leptos::prelude::*;

use super::use_sidebar;

#[component]
pub fn SidebarMenu(children: Children) -> impl IntoView {
    view! {
        <ul data-slot="sidebar-menu" data-sidebar="menu"
            class="flex w-full min-w-0 flex-col gap-0"
        >
            {children()}
        </ul>
    }
}

#[component]
pub fn SidebarMenuItem(children: Children) -> impl IntoView {
    view! {
        <li data-slot="sidebar-menu-item" data-sidebar="menu-item"
            class="group/menu-item relative"
        >
            {children()}
        </li>
    }
}

#[component]
pub fn SidebarMenuButton(
    children: Children,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    tooltip: Option<String>,
    #[prop(default = false)] is_active: bool,
) -> impl IntoView {
    let ctx = use_sidebar();
    let open = ctx.open;

    let handle_click = move |_| {
        if let Some(ref cb) = on_click {
            cb();
        }
    };

    let title_text = Signal::derive(move || {
        if !open.get() {
            tooltip.clone().unwrap_or_default()
        } else {
            String::new()
        }
    });

    view! {
        <button
            on:click=handle_click
            title=move || title_text.get()
            class="peer/menu-button flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm ring-sidebar-ring outline-hidden transition-[width,height,padding] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 group-data-[collapsible=icon]:size-8! group-data-[collapsible=icon]:p-2! data-active:bg-sidebar-accent data-active:font-medium data-active:text-sidebar-accent-foreground [&>svg]:size-4 [&>svg]:shrink-0 [&>span:last-child]:truncate"
            data-active=move || if is_active { "true" } else { "false" }
        >
            {children()}
        </button>
    }
}

#[component]
pub fn SidebarMenuSub(children: Children) -> impl IntoView {
    view! {
        <ul data-slot="sidebar-menu-sub" data-sidebar="menu-sub"
            class="mx-3.5 flex min-w-0 translate-x-px flex-col gap-1 border-l border-sidebar-border px-2.5 py-0.5 group-data-[collapsible=icon]:hidden"
        >
            {children()}
        </ul>
    }
}

#[component]
pub fn SidebarMenuSubItem(children: Children) -> impl IntoView {
    view! {
        <li data-slot="sidebar-menu-sub-item" data-sidebar="menu-sub-item"
            class="group/menu-sub-item relative"
        >
            {children()}
        </li>
    }
}

#[component]
pub fn SidebarMenuSubButton(
    children: Children,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    #[prop(default = false)] is_active: bool,
) -> impl IntoView {
    let handle_click = move |_| {
        if let Some(ref cb) = on_click {
            cb();
        }
    };

    view! {
        <button
            on:click=handle_click
            class="flex h-7 min-w-0 -translate-x-px items-center gap-2 overflow-hidden rounded-md px-2 text-sidebar-foreground ring-sidebar-ring outline-hidden group-data-[collapsible=icon]:hidden hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 data-active:bg-sidebar-accent data-active:text-sidebar-accent-foreground [&>span:last-child]:truncate w-full text-left text-sm [&>svg]:size-4 [&>svg]:shrink-0"
            data-active=move || if is_active { "true" } else { "false" }
        >
            {children()}
        </button>
    }
}
