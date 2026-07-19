use leptos::prelude::*;
use crate::shared::OverlayClose;

#[component]
pub fn SheetClose(
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
