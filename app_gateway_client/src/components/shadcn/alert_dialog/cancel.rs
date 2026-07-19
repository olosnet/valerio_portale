#![allow(dead_code)]
use leptos::prelude::*;
use crate::components::shadcn::shared::OverlayClose;

#[component]
pub fn AlertDialogCancel(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <OverlayClose class=extra>
            {children()}
        </OverlayClose>
    }
}
