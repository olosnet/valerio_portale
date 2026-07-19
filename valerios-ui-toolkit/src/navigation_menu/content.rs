use leptos::prelude::*;
use super::item::use_nav_menu_item;

#[component]
pub fn NavigationMenuContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_nav_menu_item();
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "absolute z-50 left-0 top-full mt-1 w-auto min-w-[12rem] rounded-md border bg-popover p-4 text-popover-foreground shadow-md animate-fade-in {}",
        extra,
    );

    let backdrop = move |_| ctx.open.set(false);
    let stop = move |ev: leptos::ev::MouseEvent| ev.stop_propagation();
    let escape = move |ev: leptos::ev::KeyboardEvent| { if ev.key() == "Escape" { ctx.open.set(false); } };

    move || {
        if ctx.open.get() {
            view! {
                <div on:click=backdrop class="fixed inset-0 z-40" />
                <div data-slot="navigation-menu-content" on:click=stop on:keydown=escape class=cls()>
                    {children()}
                </div>
            }.into_any()
        } else { ().into_any() }
    }
}
