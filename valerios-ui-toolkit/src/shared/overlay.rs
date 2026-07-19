use leptos::prelude::*;

#[derive(Clone)]
pub struct OverlayContext {
    pub open: RwSignal<bool>,
}

impl OverlayContext {
    pub fn new(default_open: bool) -> Self {
        Self { open: RwSignal::new(default_open) }
    }
}

pub fn use_overlay() -> OverlayContext {
    expect_context::<OverlayContext>()
}

#[component]
pub fn OverlayProvider(
    children: Children,
    open: RwSignal<bool>,
) -> impl IntoView {
    provide_context(OverlayContext { open });
    view! { {children()} }
}

#[component]
pub fn Overlay(
    children: Children,
    #[prop(default = "fixed inset-0 z-50 bg-black/80 animate-fade-in")]
    backdrop_class: &'static str,
    #[prop(default = "fixed left-[50%] top-[50%] z-50 w-full max-w-lg translate-x-[-50%] translate-y-[-50%] border bg-background p-6 shadow-lg animate-zoom-in sm:rounded-lg")]
    content_class: &'static str,
) -> impl IntoView {
    let ctx = use_overlay();
    let content = children();

    view! {
        <div data-slot="overlay-wrapper" class=move || if ctx.open.get() { "" } else { "hidden" }>
            <div
                on:click=move |_| ctx.open.set(false)
                class=backdrop_class
            />

            <div
                on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
                on:keydown=move |ev: leptos::ev::KeyboardEvent| { if ev.key() == "Escape" { ctx.open.set(false); } }
                tabindex="-1" role="dialog" aria-modal="true"
                class=content_class
            >
                {content}
            </div>
        </div>
    }
}

#[component]
pub fn OverlayClose(
    children: Children,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let ctx = use_overlay();
    view! {
        <button on:click=move |_| ctx.open.set(false) class=class type="button">
            {children()}
        </button>
    }
}
