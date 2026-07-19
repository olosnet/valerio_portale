use leptos::prelude::*;

#[component]
pub fn BreadcrumbSeparator(
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <li role="presentation" aria-hidden="true" data-slot="breadcrumb-separator" class=format!("[&>svg]:size-3.5 {}", extra)>
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4">
                <path d="m9 18 6-6-6-6"/>
            </svg>
        </li>
    }
}
