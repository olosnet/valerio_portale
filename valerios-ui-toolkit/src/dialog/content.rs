use leptos::prelude::*;
use crate::shared::Overlay;

#[component]
pub fn DialogContent(
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <Overlay
            content_class="fixed left-[50%] top-[50%] z-50 w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg animate-zoom-in sm:rounded-lg"
        >
            {children()}
        </Overlay>
    }
}
