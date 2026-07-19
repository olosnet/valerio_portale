use leptos::prelude::*;
use super::select::use_select;

#[component]
pub fn SelectContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_select();
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "absolute top-full left-0 z-50 mt-1 w-full min-w-[8rem] overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md animate-fade-in {}",
        extra,
    );

    let handle_backdrop = move |_| ctx.open.set(false);
    let handle_content_click = move |ev: leptos::ev::MouseEvent| ev.stop_propagation();
    let handle_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Escape" { ctx.open.set(false); }
    };

    move || {
        if ctx.open.get() {
            view! {
                <div on:click=handle_backdrop class="fixed inset-0 z-40" />
                <div
                    data-slot="select-content"
                    on:click=handle_content_click
                    on:keydown=handle_keydown
                    class=cls()
                >
                    {children()}
                </div>
            }.into_any()
        } else { ().into_any() }
    }
}
