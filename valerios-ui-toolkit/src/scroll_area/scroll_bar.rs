use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ScrollOrientation {
    Vertical,
    Horizontal,
}

#[component]
pub fn ScrollBar(
    #[prop(default = ScrollOrientation::Vertical)] orientation: ScrollOrientation,
) -> impl IntoView {
    match orientation {
        ScrollOrientation::Vertical => view! {
            <div data-slot="scroll-bar" role="scrollbar" aria-orientation="vertical"
                class="flex touch-none select-none transition-colors w-2.5 h-full p-0.5">
                <div class="relative flex-1 rounded-full bg-border" />
            </div>
        },
        ScrollOrientation::Horizontal => view! {
            <div data-slot="scroll-bar" role="scrollbar" aria-orientation="horizontal"
                class="flex touch-none select-none transition-colors h-2.5 w-full p-0.5">
                <div class="relative flex-1 rounded-full bg-border" />
            </div>
        },
    }
}
