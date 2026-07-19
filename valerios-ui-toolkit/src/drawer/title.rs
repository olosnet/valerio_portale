use leptos::prelude::*;

#[component]
pub fn DrawerTitle(
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div data-slot="drawer-title" class="text-lg font-semibold leading-none tracking-tight text-foreground">
            {children()}
        </div>
    }
}
