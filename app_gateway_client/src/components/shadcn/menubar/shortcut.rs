#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn MenubarShortcut(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! { <span data-slot="menubar-shortcut" class=format!("ml-auto text-xs tracking-widest text-muted-foreground {}", extra)>{children()}</span> }
}
