use leptos::prelude::*;

#[component]
pub fn MenubarGroup(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    view! { <div role="group" data-slot="menubar-group" class=class.unwrap_or("")>{children()}</div> }
}
