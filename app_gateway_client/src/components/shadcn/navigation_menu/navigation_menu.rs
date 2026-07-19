#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn NavigationMenu(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <nav data-slot="navigation-menu" class=format!("relative z-10 flex max-w-max flex-1 items-center justify-center {}", extra)>
            {children()}
        </nav>
    }
}
