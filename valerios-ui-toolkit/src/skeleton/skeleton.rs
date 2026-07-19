use leptos::prelude::*;

#[component]
pub fn Skeleton(
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("animate-pulse rounded-md bg-muted {}", extra);

    view! {
        <div data-slot="skeleton" class=cls() />
    }
}
