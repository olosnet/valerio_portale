use leptos::prelude::*;
use crate::shared::Overlay;
use super::sheet::{use_sheet_side, SheetSide};

fn sheet_side_class(side: SheetSide) -> &'static str {
    match side {
        SheetSide::Left => "left-0 top-0 h-full w-72 border-r animate-slide-in-left",
        SheetSide::Right => "right-0 top-0 h-full w-72 border-l animate-slide-in-right",
        SheetSide::Top => "top-0 left-0 w-full h-48 border-b animate-slide-in-top",
        SheetSide::Bottom => "bottom-0 left-0 w-full h-48 border-t animate-slide-in-bottom",
    }
}

#[component]
pub fn SheetContent(
    children: ChildrenFn,
) -> impl IntoView {
    let side = use_sheet_side();

    view! {
        <Overlay content_class=sheet_side_class(side)>
            {children()}
        </Overlay>
    }
}
