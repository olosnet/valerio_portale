use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
pub struct OverlayContext {
    pub open: RwSignal<bool>,
    pub closing: RwSignal<bool>,
}

impl OverlayContext {
    pub fn new(default_open: bool) -> Self {
        Self {
            open: RwSignal::new(default_open),
            closing: RwSignal::new(false),
        }
    }

    pub fn close(&self) {
        if self.closing.get() {
            return;
        }
        self.closing.set(true);
        let open = self.open;
        let closing = self.closing;
        let window = web_sys::window().unwrap();
        let cb = Closure::once(move || {
            open.set(false);
            closing.set(false);
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                200,
            )
            .unwrap();
        cb.forget();
    }

    pub fn open_overlay(&self) {
        self.open.set(true);
        self.closing.set(false);
    }
}

impl From<RwSignal<bool>> for OverlayContext {
    fn from(open: RwSignal<bool>) -> Self {
        Self {
            open,
            closing: RwSignal::new(false),
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
    let ctx: OverlayContext = open.into();

    Effect::new(move || {
        if ctx.open.get() {
            ctx.closing.set(false);
        }
    });

    provide_context(ctx);
    view! { {children()} }
}

#[component]
pub fn Overlay(
    children: Children,
    #[prop(default = "fixed inset-0 z-50 bg-black/80 animate-fade-in")]
    backdrop_class: &'static str,
    #[prop(default = "fixed left-[50%] top-[50%] z-50 w-full max-w-lg translate-x-[-50%] translate-y-[-50%] border bg-background p-6 shadow-lg animate-zoom-in sm:rounded-lg")]
    content_class: &'static str,
    #[prop(default = "animate-zoom-out")]
    exit_content_class: &'static str,
) -> impl IntoView {
    let ctx = use_overlay();
    let content = children();

    let wrapper_class = move || {
        if ctx.open.get() {
            ""
        } else if ctx.closing.get() {
            ""
        } else {
            "hidden"
        }
    };

    let backdrop_cls = move || {
        if ctx.closing.get() {
            "fixed inset-0 z-50 bg-black/80 animate-fade-out pointer-events-none"
        } else {
            backdrop_class
        }
    };

    let content_cls = move || {
        if !ctx.open.get() {
            strip_entry_anim(content_class)
        } else if ctx.closing.get() {
            format!("{} {} pointer-events-none", strip_entry_anim(content_class), exit_content_class)
        } else {
            content_class.to_string()
        }
    };

    view! {
        <div data-slot="overlay-wrapper" class=wrapper_class>
            <div
                on:click=move |_| ctx.close()
                class=backdrop_cls
            />
            <div
                on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
                on:keydown=move |ev: leptos::ev::KeyboardEvent| { if ev.key() == "Escape" { ctx.close(); } }
                tabindex="-1" role="dialog" aria-modal="true"
                class=content_cls
            >
                {content}
            </div>
        </div>
    }
}

fn strip_entry_anim(class: &str) -> String {
    class
        .split(' ')
        .filter(|w| !w.starts_with("animate-"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
pub fn OverlayClose(
    children: Children,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let ctx = use_overlay();
    view! {
        <button on:click=move |_| ctx.close() class=class type="button">
            {children()}
        </button>
    }
}
