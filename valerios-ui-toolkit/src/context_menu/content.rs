use leptos::prelude::*;
use super::context_menu::use_context_menu;

#[component]
pub fn ContextMenuContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_context_menu();
    let extra = class.unwrap_or("");

    let handle_backdrop = move |_| ctx.open.set(false);
    let handle_content_click = move |ev: leptos::ev::MouseEvent| ev.stop_propagation();
    let handle_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Escape" { ctx.open.set(false); }
    };

    move || {
        if ctx.open.get() {
            let left = ctx.x.get();
            let top = ctx.y.get();
            let style = format!("left:{}px; top:{}px;", left, top);

            view! {
                <div on:click=handle_backdrop class="fixed inset-0 z-40" />
                <div
                    data-slot="context-menu-content"
                    on:click=handle_content_click
                    on:keydown=handle_keydown
                    role="menu"
                    style=style
                    class=format!("fixed z-50 min-w-[8rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md animate-fade-in {}", extra)
                >
                    {children()}
                </div>
            }.into_any()
        } else { ().into_any() }
    }
}
