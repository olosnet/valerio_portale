#![allow(dead_code)]
use leptos::prelude::*;
use valerios_ui_toolkit::icon::Icon;
use valerios_ui_toolkit::sidebar::{use_sidebar, SidebarProvider};

use crate::modules::base::components::app_sidebar::AppSidebar;

#[component]
fn LayoutHeader() -> impl IntoView {
    let sidebar_open = use_sidebar().open;

    view! {
        <div data-slot="layout-header"
            class="sticky top-0 z-10 flex items-center gap-2 border-b border-border bg-background/80 backdrop-blur-sm px-4 py-1.5 min-h-[40px]">
            <button
                on:click=move |_| sidebar_open.update(|v| *v = !*v)
                class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground h-7 w-7"
                title=move || if sidebar_open.get() { "Chiudi sidebar" } else { "Apri sidebar" }
            >
                {move || if sidebar_open.get() { Icon::PanelLeftOpen.render() } else { Icon::PanelLeft.render() }}
            </button>
            <div class="text-xs text-muted-foreground">
                {move || {
                    // mostra il percorso corrente se serve
                    ""
                }}
            </div>
        </div>
    }
}

pub fn with_layout(content: impl IntoView + 'static) -> impl IntoView {
    view! {
        <SidebarProvider>
            <AppSidebar/>
            <div class="flex flex-1 flex-col min-w-0">
                <LayoutHeader/>
                <main class="flex-1 bg-background p-8 overflow-auto">
                    {content}
                </main>
            </div>
        </SidebarProvider>
    }
}
