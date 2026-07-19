use leptos::prelude::*;

#[component]
pub fn DrawerFooter(
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div data-slot="drawer-footer" class="flex flex-col-reverse sm:flex-row sm:justify-end sm:gap-x-2 mt-4">
            {children()}
        </div>
    }
}
