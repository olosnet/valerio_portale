use leptos::prelude::*;

#[component]
pub fn DropdownMenuShortcut(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <span data-slot="dropdown-menu-shortcut" class=format!("ml-auto text-xs tracking-widest text-muted-foreground {}", extra)>
            {children()}
        </span>
    }
}
