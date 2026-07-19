#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn DrawerHeader(
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div data-slot="drawer-header" class="flex flex-col gap-y-1.5 text-center sm:text-left">
            {children()}
        </div>
    }
}
