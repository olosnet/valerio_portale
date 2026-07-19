use leptos::prelude::*;

#[derive(Clone)]
pub struct AvatarContext {
    pub has_image: RwSignal<bool>,
}

impl AvatarContext {
    pub fn new() -> Self {
        Self {
            has_image: RwSignal::new(true),
        }
    }
}

pub fn use_avatar() -> AvatarContext {
    expect_context::<AvatarContext>()
}

#[component]
pub fn AvatarImage(
    src: &'static str,
    #[prop(default = "")] alt: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_avatar();
    let extra = class.unwrap_or("");
    let cls = move || format!("aspect-square h-full w-full {}", extra);

    let on_error = move |_| {
        ctx.has_image.set(false);
    };

    view! {
        {move || if ctx.has_image.get() {
            view! {
                <img src=src alt=alt class=cls() on:error=on_error />
            }.into_any()
        } else {
            ().into_any()
        }}
    }
}
