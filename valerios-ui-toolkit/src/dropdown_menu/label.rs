use leptos::prelude::*;

#[component]
pub fn DropdownMenuLabel(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("px-2 py-1.5 text-sm font-semibold text-popover-foreground {}", extra);

    view! {
        <div data-slot="dropdown-menu-label" class=cls()>
            {children()}
        </div>
    }
}
