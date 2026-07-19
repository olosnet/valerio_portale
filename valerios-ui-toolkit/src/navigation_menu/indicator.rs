use leptos::prelude::*;

#[component]
pub fn NavigationMenuIndicator(
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <div data-slot="navigation-menu-indicator" class=format!("absolute bottom-0 left-0 right-0 flex justify-center {}", extra)>
            <div class="size-2 rotate-45 rounded-tl-sm border bg-popover" />
        </div>
    }
}
