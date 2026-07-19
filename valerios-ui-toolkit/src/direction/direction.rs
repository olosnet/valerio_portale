use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum TextDirection {
    Ltr,
    Rtl,
}

#[derive(Clone)]
pub struct DirectionContext {
    pub dir: TextDirection,
}

impl DirectionContext {
    pub fn new(dir: TextDirection) -> Self { Self { dir } }
}

pub fn use_direction() -> DirectionContext {
    expect_context::<DirectionContext>()
}

#[component]
pub fn DirectionProvider(
    children: ChildrenFn,
    #[prop(default = TextDirection::Rtl)] dir: TextDirection,
) -> impl IntoView {
    provide_context(DirectionContext { dir });
    let dir_attr = match dir { TextDirection::Rtl => "rtl", _ => "ltr" };

    view! {
        <div dir=dir_attr data-slot="direction-provider">
            {children()}
        </div>
    }
}
