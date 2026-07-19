#![allow(dead_code)]
use leptos::prelude::*;
use crate::components::shadcn::shared::OverlayProvider;

#[derive(Clone, Copy, PartialEq)]
pub enum SheetSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[component]
pub fn Sheet(
    children: ChildrenFn,
    open: RwSignal<bool>,
    #[prop(default = SheetSide::Right)] side: SheetSide,
) -> impl IntoView {
    provide_context(side);

    view! {
        <OverlayProvider open=open>
            {children()}
        </OverlayProvider>
    }
}

pub fn use_sheet_side() -> SheetSide {
    expect_context::<SheetSide>()
}
