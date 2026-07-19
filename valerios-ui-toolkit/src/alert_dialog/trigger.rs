use leptos::prelude::*;
use crate::shared::use_overlay;

#[component]
pub fn AlertDialogTrigger(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_overlay();
    let extra = class.unwrap_or("");

    view! {
        <div
            data-slot="alert-dialog-trigger"
            on:click=move |_| ctx.open.set(true)
            class=extra
        >
            {children()}
        </div>
    }
}
