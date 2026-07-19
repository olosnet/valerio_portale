use leptos::prelude::*;

use super::use_sidebar;

#[component]
pub fn SidebarRail() -> impl IntoView {
    let ctx = use_sidebar();

    view! {
        <button
            on:click=move |_| ctx.toggle()
            title="Attiva/Disattiva barra laterale"
            class="absolute inset-y-0 z-20 hidden w-4 -right-4
                   after:absolute after:inset-y-0 after:left-1/2 after:w-[2px]
                   hover:after:bg-sidebar-border
                   cursor-w-resize
                   sm:flex"
        />
    }
}
