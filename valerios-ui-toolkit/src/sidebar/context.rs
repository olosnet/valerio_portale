use leptos::prelude::*;
use wasm_bindgen::prelude::*;

// Breakpoint allineato a Tailwind `lg` (1024px)
const MOBILE_BREAKPOINT: f64 = 1024.0;

fn is_mobile_window() -> bool {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .map(|w| w.as_f64().unwrap_or(1280.0) < MOBILE_BREAKPOINT)
        .unwrap_or(false)
}

#[derive(Clone)]
pub struct SidebarContext {
    pub open: RwSignal<bool>,
    pub open_mobile: RwSignal<bool>,
    pub is_mobile: RwSignal<bool>,
}

impl SidebarContext {
    pub fn new(default_open: bool) -> Self {
        Self {
            open: RwSignal::new(default_open),
            open_mobile: RwSignal::new(false),
            is_mobile: RwSignal::new(is_mobile_window()),
        }
    }

    pub fn toggle(&self) {
        if self.is_mobile.get_untracked() {
            self.open_mobile.update(|v| *v = !*v);
        } else {
            self.open.update(|v| *v = !*v);
        }
    }
}

#[component]
pub fn SidebarProvider(
    children: Children,
    #[prop(default = true)] default_open: bool,
) -> impl IntoView {
    let ctx = SidebarContext::new(default_open);
    provide_context(ctx.clone());

    // Aggiorna is_mobile al resize della finestra
    let is_mobile = ctx.is_mobile;
    let open_mobile = ctx.open_mobile;
    if let Some(window) = web_sys::window() {
        let cb = Closure::<dyn Fn()>::new(move || {
            let mobile = is_mobile_window();
            is_mobile.set(mobile);
            // Chiudi mobile sheet se torniamo a desktop
            if !mobile {
                open_mobile.set(false);
            }
        });
        let _ = window.add_event_listener_with_callback("resize", cb.as_ref().unchecked_ref());
        cb.forget(); // Vive per sempre (lifetime del SidebarProvider)
    }

    view! {
        <div data-slot="sidebar-wrapper"
            class="group/sidebar-wrapper flex min-h-svh w-full"
            style="--sidebar-width: 16rem; --sidebar-width-icon: 3rem;"
        >
            {children()}
        </div>
    }
}

pub fn use_sidebar() -> SidebarContext {
    expect_context::<SidebarContext>()
}
