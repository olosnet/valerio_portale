use leptos::prelude::*;
use crate::shared::Overlay;

#[component]
pub fn DrawerContent(
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <Overlay
            content_class="fixed bottom-0 left-0 right-0 z-50 mt-24 flex h-auto flex-col rounded-t-[10px] border bg-background p-6 shadow-lg animate-slide-in-bottom max-h-[90vh]"
        >
            <div class="mx-auto mt-2 h-2 w-[100px] rounded-full bg-muted" />
            {children()}
        </Overlay>
    }
}
