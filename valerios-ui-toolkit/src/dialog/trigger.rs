use leptos::prelude::*;
use crate::shared::use_overlay;

#[component]
pub fn DialogTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_overlay();
    let extra = class.unwrap_or("");
    let cls = move || format!("{}", extra);

    view! {
        <div
            data-slot="dialog-trigger"
            on:click=move |_| ctx.open.set(true)
            class=cls()
        >
            {children()}
        </div>
    }
}
