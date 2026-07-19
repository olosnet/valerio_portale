use leptos::prelude::*;

#[component]
pub fn CommandShortcut(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <span data-slot="command-shortcut" class=format!("ml-auto text-xs tracking-widest text-muted-foreground {}", extra)>
            {children()}
        </span>
    }
}
