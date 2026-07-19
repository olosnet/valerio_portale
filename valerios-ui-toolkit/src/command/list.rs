use leptos::prelude::*;

#[component]
pub fn CommandList(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("max-h-[300px] overflow-y-auto overflow-x-hidden {}", extra);

    view! {
        <div data-slot="command-list" role="listbox" class=cls()>
            {children()}
        </div>
    }
}
