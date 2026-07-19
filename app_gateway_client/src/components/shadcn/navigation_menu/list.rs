#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn NavigationMenuList(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("group flex flex-1 list-none items-center justify-center gap-1 {}", extra);

    view! {
        <ul data-slot="navigation-menu-list" class=cls()>
            {children()}
        </ul>
    }
}
