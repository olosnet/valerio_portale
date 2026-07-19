use leptos::prelude::*;

use super::use_sidebar;

#[component]
pub fn Sidebar(
    children: Children,
    #[prop(default = "icon")] collapsible: &'static str,
    #[prop(default = "desktop")] variant: &'static str,
) -> impl IntoView {
    let ctx = use_sidebar();

    if variant == "mobile" {
        // Mobile: niente hidden/fixed, layout semplice dentro Sheet
        view! {
            <div
                class="group flex h-full w-full flex-col bg-sidebar text-sidebar-foreground"
                data-state="expanded"
                data-slot="sidebar"
            >
                {children()}
            </div>
        }.into_any()
    } else {
        // Desktop: layout originale con hidden md:block, fixed, gap spacer
        view! {
            <div
                class="group peer hidden text-sidebar-foreground md:block"
                data-state=move || if ctx.open.get() { "expanded" } else { "collapsed" }
                data-collapsible=move || if ctx.open.get() { "" } else { collapsible }
                data-slot="sidebar"
            >
                <div data-slot="sidebar-gap"
                    class="relative w-(--sidebar-width) bg-transparent transition-[width] duration-200 ease-linear group-data-[collapsible=icon]:w-(--sidebar-width-icon)"
                />
                <div data-slot="sidebar-container"
                    class="fixed inset-y-0 z-10 hidden h-svh w-(--sidebar-width) transition-[left,right,width] duration-200 ease-linear left-0 md:flex group-data-[collapsible=icon]:w-(--sidebar-width-icon) border-r border-sidebar-border"
                >
                    <div
                        data-sidebar="sidebar"
                        data-slot="sidebar-inner"
                        class="flex size-full flex-col bg-sidebar"
                    >
                        {children()}
                    </div>
                </div>
            </div>
        }.into_any()
    }
}

#[component]
pub fn SidebarHeader(children: Children) -> impl IntoView {
    view! {
        <div data-slot="sidebar-header" data-sidebar="header"
            class="flex flex-col gap-2 p-2"
        >
            {children()}
        </div>
    }
}

#[component]
pub fn SidebarContent(children: Children) -> impl IntoView {
    view! {
        <div data-slot="sidebar-content" data-sidebar="content"
            class="flex min-h-0 flex-1 flex-col gap-0 overflow-auto group-data-[collapsible=icon]:overflow-hidden"
        >
            {children()}
        </div>
    }
}

#[component]
pub fn SidebarFooter(children: Children) -> impl IntoView {
    view! {
        <div data-slot="sidebar-footer" data-sidebar="footer"
            class="flex flex-col gap-2 p-2 border-t border-sidebar-border"
        >
            {children()}
        </div>
    }
}
