use leptos::prelude::*;
use crate::shared::OverlayProvider;

#[component]
pub fn Dialog(
    children: ChildrenFn,
    open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <OverlayProvider open=open>
            {children()}
        </OverlayProvider>
    }
}
