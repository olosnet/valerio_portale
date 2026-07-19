use leptos::prelude::*;

#[component]
pub fn AlertDescription(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("text-sm [&_p]:leading-relaxed {}", extra);

    view! {
        <div data-slot="alert-description" class=cls()>
            {children()}
        </div>
    }
}
