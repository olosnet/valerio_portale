use leptos::prelude::*;

use crate::components::sidebar::Sidebar;

pub fn with_layout(content: impl IntoView + 'static) -> impl IntoView {
    view! {
        <div class="flex min-h-svh w-full bg-background">
            <Sidebar/>
            <main class="relative flex w-full flex-1 flex-col bg-background p-8 overflow-auto">
                {content}
            </main>
        </div>
    }
}
