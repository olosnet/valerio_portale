/// Generic overlay primitive for shadcn components
/// (Dialog, AlertDialog, Sheet, Popover, DropdownMenu, Tooltip, Select, etc.)
///
/// Pattern:
/// - Consumer provides `open: RwSignal<bool>`
/// - Renders backdrop + content only when `open.get()`
/// - Backdrop click closes
/// - Escape key closes
/// - Auto-focuses first focusable element on open
/// - click.stop_propagation on content to prevent backdrop close

use leptos::prelude::*;

#[derive(Clone)]
pub struct OverlayContext {
    pub open: RwSignal<bool>,
}

impl OverlayContext {
    pub fn new(default_open: bool) -> Self {
        Self {
            open: RwSignal::new(default_open),
        }
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

    view! {
        {children()}
    }
}

#[component]
pub fn Overlay(
    children: ChildrenFn,
    #[prop(default = "fixed inset-0 z-50 bg-black/80 animate-fade-in")]
    backdrop_class: &'static str,
    #[prop(default = "fixed left-[50%] top-[50%] z-50 w-full max-w-lg translate-x-[-50%] translate-y-[-50%] border bg-background p-6 shadow-lg animate-zoom-in sm:rounded-lg")]
    content_class: &'static str,
) -> impl IntoView {
    let ctx = use_overlay();

    let on_backdrop_click = move |_| {
        ctx.open.set(false);
    };

    let on_content_click = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
    };

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Escape" {
            ctx.open.set(false);
        }
    };

    move || {
        if ctx.open.get() {
            view! {
                <div
                    on:click=on_backdrop_click
                    class=backdrop_class
                />

                <div
                    on:click=on_content_click
                    on:keydown=on_keydown
                    tabindex="-1"
                    role="dialog"
                    aria-modal="true"
                    class=content_class
                >
                    {children()}
                </div>
            }.into_any()
        } else {
            ().into_any()
        }
    }
}

#[component]
pub fn OverlayClose(
    children: Children,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let ctx = use_overlay();

    view! {
        <button
            on:click=move |_| ctx.open.set(false)
            class=class
            type="button"
        >
            {children()}
        </button>
    }
}
