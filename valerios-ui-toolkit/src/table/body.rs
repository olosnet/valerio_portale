use leptos::prelude::*;

#[component]
pub fn TableBody(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("[&_tr:last-child]:border-0 {}", extra);

    view! {
        <tbody data-slot="table-body" class=cls()>
            {children()}
        </tbody>
    }
}
