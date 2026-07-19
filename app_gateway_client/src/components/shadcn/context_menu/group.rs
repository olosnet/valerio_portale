#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn ContextMenuGroup(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <div role="group" data-slot="context-menu-group" class=extra>
            {children()}
        </div>
    }
}
