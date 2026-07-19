use leptos::prelude::*;
use super::popover::use_popover;

#[component]
pub fn PopoverContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = "bottom")] side: &'static str,
) -> impl IntoView {
    let ctx = use_popover();
    let extra = class.unwrap_or("");

    let (side_class, align_class) = match side {
        "top" => ("bottom-full left-1/2 -translate-x-1/2 mb-2", ""),
        "bottom" => ("top-full left-1/2 -translate-x-1/2 mt-2", ""),
        "left" => ("right-full top-1/2 -translate-y-1/2 mr-2", ""),
        "right" => ("left-full top-1/2 -translate-y-1/2 ml-2", ""),
        _ => ("top-full left-0 mt-1", ""),
    };

    let cls = move || format!(
        "absolute z-50 w-72 rounded-md border bg-popover p-4 text-popover-foreground shadow-md animate-fade-in {} {} {}",
        side_class, align_class, extra,
    );

    let handle_backdrop = move |_| ctx.open.set(false);
    let handle_content_click = move |ev: leptos::ev::MouseEvent| ev.stop_propagation();
    let handle_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Escape" { ctx.open.set(false); }
    };

    move || {
        if ctx.open.get() {
            view! {
                <div
                    on:click=handle_backdrop
                    class="fixed inset-0 z-40"
                />
                <div
                    data-slot="popover-content"
                    on:click=handle_content_click
                    on:keydown=handle_keydown
                    class=cls()
                >
                    {children()}
                </div>
            }.into_any()
        } else {
            ().into_any()
        }
    }
}
