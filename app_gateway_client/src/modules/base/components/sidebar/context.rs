use leptos::prelude::*;

#[derive(Clone)]
pub struct SidebarContext {
    pub open: RwSignal<bool>,
}

impl SidebarContext {
    pub fn new(default_open: bool) -> Self {
        Self {
            open: RwSignal::new(default_open),
        }
    }

    pub fn toggle(&self) {
        self.open.update(|v| *v = !*v);
    }
}

#[component]
pub fn SidebarProvider(
    children: Children,
    #[prop(default = true)] default_open: bool,
) -> impl IntoView {
    let ctx = SidebarContext::new(default_open);
    provide_context(ctx);

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
