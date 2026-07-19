use leptos::prelude::*;

#[component]
pub fn DrawerDescription(
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div data-slot="drawer-description" class="text-sm text-muted-foreground">
            {children()}
        </div>
    }
}
