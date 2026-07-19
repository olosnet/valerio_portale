use leptos::prelude::*;
use super::command::use_command;

#[component]
pub fn CommandEmpty(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_command();
    let extra = class.unwrap_or("");

    move || {
        if !ctx.query.get().is_empty() {
            view! {
                <div data-slot="command-empty" class=format!("py-6 text-center text-sm {}", extra)>
                    {children()}
                </div>
            }.into_any()
        } else { ().into_any() }
    }
}
