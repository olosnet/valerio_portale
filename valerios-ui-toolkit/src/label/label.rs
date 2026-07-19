use leptos::prelude::*;

#[component]
pub fn Label(
    children: ChildrenFn,
    #[prop(optional)] html_for: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let base = "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70";
    let extra = class.unwrap_or("");
    let cls = move || format!("{} {}", base, extra);

    view! {
        <label
            for=html_for
            class=cls()
        >
            {children()}
        </label>
    }
}
