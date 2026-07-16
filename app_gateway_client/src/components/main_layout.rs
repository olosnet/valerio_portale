use leptos::prelude::*;

use crate::components::sidebar::Sidebar;

pub fn with_layout(content: impl IntoView + 'static) -> impl IntoView {
    view! {
        <div class="flex min-h-screen bg-secondary">
            <Sidebar/>
            <main class="flex-1 p-8 overflow-auto">
                {content}
            </main>
        </div>
    }
}
