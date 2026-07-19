use leptos::prelude::*;

#[component]
pub fn TableHeader(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("[&_tr]:border-b {}", extra);

    view! {
        <thead data-slot="table-header" class=cls()>
            {children()}
        </thead>
    }
}
