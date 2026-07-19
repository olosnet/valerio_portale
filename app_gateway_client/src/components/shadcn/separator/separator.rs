#![allow(dead_code)]
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum SeparatorOrientation {
    Horizontal,
    Vertical,
}

#[component]
pub fn Separator(
    #[prop(default = SeparatorOrientation::Horizontal)] orientation: SeparatorOrientation,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let (orient_class, orient_attr) = match orientation {
        SeparatorOrientation::Horizontal => ("h-px w-full", "horizontal"),
        SeparatorOrientation::Vertical => ("h-full w-px", "vertical"),
    };
    let cls = move || format!("shrink-0 bg-border {} {}", orient_class, extra);

    view! {
        <div
            data-slot="separator"
            role="separator"
            aria-orientation=orient_attr
            class=cls()
        />
    }
}
