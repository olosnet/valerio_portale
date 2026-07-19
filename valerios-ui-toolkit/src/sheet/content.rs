use leptos::prelude::*;
use crate::shared::Overlay;
use super::sheet::{use_sheet_side, SheetSide};

fn sheet_side_class(side: SheetSide) -> &'static str {
    match side {
        SheetSide::Left => "fixed left-0 top-0 z-50 h-full w-72 border-r border-border bg-sidebar shadow-lg animate-slide-in-left",
        SheetSide::Right => "fixed right-0 top-0 z-50 h-full w-72 border-l border-border bg-sidebar shadow-lg animate-slide-in-right",
        SheetSide::Top => "fixed top-0 left-0 z-50 w-full h-48 border-b border-border bg-background shadow-lg animate-slide-in-top",
        SheetSide::Bottom => "fixed bottom-0 left-0 z-50 w-full h-48 border-t border-border bg-background shadow-lg animate-slide-in-bottom",
    }
}

#[component]
pub fn SheetContent(
    children: Children,
) -> impl IntoView {
    let side = use_sheet_side();

    view! {
        <Overlay content_class=sheet_side_class(side)>
            {children()}
        </Overlay>
    }
}
