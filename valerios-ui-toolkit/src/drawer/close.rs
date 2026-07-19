use leptos::prelude::*;
use crate::shared::OverlayClose;

#[component]
pub fn DrawerClose(
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <OverlayClose>
            {children()}
        </OverlayClose>
    }
}
