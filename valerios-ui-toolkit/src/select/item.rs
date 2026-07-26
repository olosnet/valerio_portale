use super::select::use_select;
use leptos::prelude::*;

#[component]
pub fn SelectItem(
    children: ChildrenFn,
    value: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_select();
    let extra = class.unwrap_or("");
    let cls = move || {
        // let selected = ctx.value.get() == value;
        format!(
            "relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=selected]:bg-accent data-[state=selected]:text-accent-foreground hover:bg-accent hover:text-accent-foreground {}",
            extra,
        )
    };

    let handle_click = move |_| {
        ctx.value.set(value.to_string());
        if let Some(ref cb) = ctx.on_change {
            cb(value.to_string());
        }
        ctx.open.set(false);
    };

    view! {
        <div
            role="option"
            data-slot="select-item"
            data-state=move || if ctx.value.get() == value { "selected" } else { "unselected" }
            on:click=handle_click
            class=cls()
        >
            <span class="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
                {move || if ctx.value.get() == value {
                    view! {
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4">
                            <path d="M20 6 9 17l-5-5"/>
                        </svg>
                    }.into_any()
                } else { ().into_any() }}
            </span>
            {children()}
        </div>
    }
}
