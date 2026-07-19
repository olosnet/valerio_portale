use leptos::prelude::*;

use valerios_ui_toolkit::sidebar::SidebarProvider;

use crate::modules::base::components::app_sidebar::AppSidebar;

pub fn with_layout(content: impl IntoView + 'static) -> impl IntoView {
    view! {
        <SidebarProvider>
            <AppSidebar/>
            <main class="relative flex w-full flex-1 flex-col bg-background p-8 overflow-auto">
                {content}
            </main>
        </SidebarProvider>
    }
}
