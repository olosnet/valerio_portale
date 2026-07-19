use leptos::prelude::*;

#[component]
pub fn Spinner(
    #[prop(default = 16)] size: u16,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <div data-slot="spinner" role="status" class=format!("animate-spin text-muted-foreground {}", extra)>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style=format!("width:{}px; height:{}px;", size, size)>
                <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
            </svg>
        </div>
    }
}
