#![allow(dead_code)]
use leptos::prelude::*;
use super::image::AvatarContext;

#[component]
pub fn AvatarFallback(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx: AvatarContext = expect_context();
    let extra = class.unwrap_or("");
    let cls = move || format!("flex h-full w-full items-center justify-center rounded-full bg-muted {}", extra);

    view! {
        {move || if !ctx.has_image.get() {
            view! {
                <span data-slot="avatar-fallback" class=cls()>
                    {children()}
                </span>
            }.into_any()
        } else {
            ().into_any()
        }}
    }
}
