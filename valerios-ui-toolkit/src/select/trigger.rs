use leptos::prelude::*;
use super::select::use_select;

#[component]
pub fn SelectTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_select();
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1 {}",
        extra,
    );

    let handle_click = move |_| ctx.open.update(|v| *v = !*v);

    view! {
        <button type="button" data-slot="select-trigger" on:click=handle_click class=cls()>
            {children()}
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4 shrink-0 opacity-50">
                <path d="m6 9 6 6 6-6"/>
            </svg>
        </button>
    }
}
