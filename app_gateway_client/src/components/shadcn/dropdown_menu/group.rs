#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn DropdownMenuGroup(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <div role="group" data-slot="dropdown-menu-group" class=extra>
            {children()}
        </div>
    }
}
