use leptos::prelude::*;

#[component]
pub fn CardTitle(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("text-2xl font-semibold leading-none tracking-tight {}", extra);

    view! {
        <div data-slot="card-title" class=cls()>
            {children()}
        </div>
    }
}
