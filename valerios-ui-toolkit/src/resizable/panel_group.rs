use leptos::prelude::*;

#[derive(Clone)]
pub struct ResizableContext {
    pub sizes: RwSignal<Vec<f64>>,
    pub direction: &'static str,
    pub panel_counter: RwSignal<usize>,
    pub handle_counter: RwSignal<usize>,
}

impl ResizableContext {
    pub fn new(sizes: RwSignal<Vec<f64>>, direction: &'static str) -> Self {
        Self {
            sizes,
            direction,
            panel_counter: RwSignal::new(0),
            handle_counter: RwSignal::new(0),
        }
    }
}

pub fn use_resizable() -> ResizableContext {
    expect_context::<ResizableContext>()
}

#[component]
pub fn ResizablePanelGroup(
    children: ChildrenFn,
    #[prop(default = "horizontal")] direction: &'static str,
) -> impl IntoView {
    let sizes: RwSignal<Vec<f64>> = RwSignal::new(Vec::new());
    let ctx = ResizableContext::new(sizes, direction);

    provide_context(ctx.clone());

    let flex_dir = match direction {
        "vertical" => "flex-col",
        _ => "flex-row",
    };

    view! {
        <div data-slot="resizable-panel-group"
            class=format!("flex h-full w-full {}", flex_dir)
            style="min-height: 0; min-width: 0;"
        >
            {children()}
        </div>
    }
}
