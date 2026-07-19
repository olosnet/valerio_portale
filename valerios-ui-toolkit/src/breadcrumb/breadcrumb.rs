use leptos::prelude::*;

#[component]
pub fn Breadcrumb(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <nav aria-label="breadcrumb" data-slot="breadcrumb" class=extra>
            {children()}
        </nav>
    }
}
