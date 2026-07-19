use leptos::prelude::*;

#[component]
pub fn SidebarGroup(children: Children) -> impl IntoView {
    view! {
        <div data-slot="sidebar-group" data-sidebar="group"
            class="relative flex w-full min-w-0 flex-col p-2"
        >
            {children()}
        </div>
    }
}

#[component]
pub fn SidebarGroupLabel(children: Children) -> impl IntoView {
    view! {
        <div data-slot="sidebar-group-label" data-sidebar="group-label"
            class="flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-sidebar-foreground/70 ring-sidebar-ring outline-hidden transition-[margin,opacity] duration-200 ease-linear group-data-[collapsible=icon]:-mt-8 group-data-[collapsible=icon]:opacity-0 focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0"
        >
            {children()}
        </div>
    }
}

#[component]
pub fn SidebarGroupContent(children: Children) -> impl IntoView {
    view! {
        <div data-slot="sidebar-group-content" data-sidebar="group-content"
            class="w-full text-sm"
        >
            {children()}
        </div>
    }
}
