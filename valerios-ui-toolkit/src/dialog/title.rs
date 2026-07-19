use leptos::prelude::*;

#[component]
pub fn DialogTitle(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("text-lg font-semibold leading-none tracking-tight {}", extra);

    view! {
        <div data-slot="dialog-title" class=cls()>
            {children()}
        </div>
    }
}
