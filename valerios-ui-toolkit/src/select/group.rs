use leptos::prelude::*;

#[component]
pub fn SelectGroup(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <div data-slot="select-group" class=extra>
            {children()}
        </div>
    }
}
