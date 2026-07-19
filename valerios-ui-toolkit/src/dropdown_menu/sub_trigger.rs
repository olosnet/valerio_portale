use leptos::prelude::*;
use super::sub::use_menu_sub;

#[component]
pub fn DropdownMenuSubTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_menu_sub();
    let extra = class.unwrap_or("");

    let handle = move |_| ctx.open.update(|v| *v = !*v);

    view! {
        <div
            data-slot="dropdown-menu-sub-trigger"
            on:click=handle
            class=format!(
                "relative flex cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-sm ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&>svg]:size-4 [&>svg]:shrink-0 {}",
                extra,
            )
        >
            {children()}
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="ml-auto size-4">
                <path d="m9 18 6-6-6-6"/>
            </svg>
        </div>
    }
}
