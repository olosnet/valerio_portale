use leptos::prelude::*;

#[component]
pub fn SelectLabel(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("py-1.5 pl-8 pr-2 text-sm font-semibold {}", extra);

    view! {
        <div data-slot="select-label" class=cls()>
            {children()}
        </div>
    }
}
