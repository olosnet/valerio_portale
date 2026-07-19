use leptos::prelude::*;

#[component]
pub fn ContextMenuLabel(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("px-2 py-1.5 text-sm font-semibold {}", extra);

    view! {
        <div data-slot="context-menu-label" class=cls()>
            {children()}
        </div>
    }
}
