use leptos::prelude::*;

#[component]
pub fn MenubarLabel(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! { <div data-slot="menubar-label" class=format!("px-2 py-1.5 text-sm font-semibold {}", extra)>{children()}</div> }
}
