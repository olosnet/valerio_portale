use leptos::prelude::*;

#[component]
pub fn FieldError(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <p data-slot="field-error" class=format!("text-sm font-medium text-destructive {}", extra)>
            {children()}
        </p>
    }
}
