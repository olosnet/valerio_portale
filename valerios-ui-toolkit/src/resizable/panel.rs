use leptos::prelude::*;
use super::panel_group::use_resizable;

#[component]
pub fn ResizablePanel(
    children: ChildrenFn,
    #[prop(default = 25.0)] default_size: f64,
    #[prop(default = 10.0)] min_size: f64,
    #[prop(default = 90.0)] max_size: f64,
) -> impl IntoView {
    let ctx = use_resizable();

    let idx = ctx.panel_counter.get();
    ctx.panel_counter.set(idx + 1);

    // Initialize size if this is a new panel (not yet in the vec)
    if idx >= ctx.sizes.get().len() {
        ctx.sizes.update(|s| s.push(default_size));
    }

    let size = Signal::derive(move || {
        let sizes = ctx.sizes.get();
        sizes.get(idx).copied().unwrap_or(default_size)
    });

    let flex_basis = move || format!("{}%", size.get());
    let min_basis = move || format!("{}%", min_size);

    let is_horizontal = ctx.direction == "horizontal";

    view! {
        <div data-slot="resizable-panel"
            style=move || format!(
                "flex-basis: {}; min-{}: {}; overflow: auto;",
                flex_basis(),
                if is_horizontal { "width" } else { "height" },
                min_basis(),
            )
            class="flex-1"
        >
            {children()}
        </div>
    }
}
