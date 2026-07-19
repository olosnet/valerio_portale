use leptos::prelude::*;

#[component]
pub fn AlertTitle(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("mb-1 font-medium leading-none tracking-tight {}", extra);

    view! {
        <h5 data-slot="alert-title" class=cls()>
            {children()}
        </h5>
    }
}
