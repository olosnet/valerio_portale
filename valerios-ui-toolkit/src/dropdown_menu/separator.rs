use leptos::prelude::*;

#[component]
pub fn DropdownMenuSeparator(
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <div role="separator" data-slot="dropdown-menu-separator" class=format!("-mx-1 my-1 h-px bg-border {}", extra) />
    }
}
