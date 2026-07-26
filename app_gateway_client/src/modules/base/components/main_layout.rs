#![allow(dead_code)]
use leptos::prelude::*;
use valerios_ui_toolkit::icon::Icon;
use valerios_ui_toolkit::sidebar::{use_sidebar, SidebarProvider};
use valerios_ui_toolkit::theme::use_theme;
use wasm_bindgen::prelude::*;

use crate::modules::base::components::app_sidebar::AppSidebar;

#[component]
fn LayoutHeader() -> impl IntoView {
    let sctx = use_sidebar();
    let theme = use_theme();
    let is_mobile = sctx.is_mobile;
    let open = sctx.open;
    let open_mobile = sctx.open_mobile;

    view! {
        <div data-slot="layout-header"
            class="sticky top-0 z-10 flex items-center justify-between border-b border-border bg-background/80 backdrop-blur-sm px-4 py-1.5 min-h-[40px]">
            <button
                on:click=move |_| sctx.toggle()
                class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground h-7 w-7"
                title=move || if is_mobile.get() {
                    if open_mobile.get() { "Chiudi menu" } else { "Apri menu" }
                } else {
                    if open.get() { "Chiudi sidebar" } else { "Apri sidebar" }
                }
            >
                {move || if is_mobile.get() {
                    if open_mobile.get() { Icon::X.render() } else { Icon::Menu.render() }
                } else {
                    if open.get() { Icon::PanelLeftOpen.render() } else { Icon::PanelLeft.render() }
                }}
            </button>
            <button
                on:click=move |_| theme.toggle_dark()
                class="inline-flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-accent h-7 w-7 ring-offset-background transition-colors"
                title=move || if theme.dark.get() { "Tema chiaro" } else { "Tema scuro" }
            >
                {move || if theme.dark.get() { Icon::Sun.render() } else { Icon::Moon.render() }}
            </button>
        </div>
    }
}

pub fn with_layout(content: impl IntoView + 'static) -> impl IntoView {
    let mounted = RwSignal::new(false);

    let cb = {
        let mounted = mounted;
        move || mounted.set(true)
    };
    let window = web_sys::window().unwrap();
    let closure = Closure::once_into_js(cb);
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            10,
        )
        .unwrap();

    view! {
        <SidebarProvider>
            <AppSidebar/>
            <div class="flex flex-1 flex-col min-w-0">
                <LayoutHeader/>
                <main class="flex-1 bg-background p-8 overflow-auto opacity-0 transition-opacity duration-300"
                      class:opacity-100=move || mounted.get()>
                    {content}
                </main>
            </div>
        </SidebarProvider>
    }
}
