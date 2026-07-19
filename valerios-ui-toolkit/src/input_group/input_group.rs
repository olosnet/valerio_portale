use leptos::prelude::*;

#[component]
pub fn InputGroup(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("flex items-center gap-2 {}", extra);
    view! { <div data-slot="input-group" class=cls()>{children()}</div> }
}
