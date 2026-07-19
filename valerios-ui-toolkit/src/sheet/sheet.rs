use leptos::prelude::*;
use crate::shared::OverlayProvider;

#[derive(Clone, Copy, PartialEq)]
pub enum SheetSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[component]
pub fn Sheet(
    children: Children,
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
