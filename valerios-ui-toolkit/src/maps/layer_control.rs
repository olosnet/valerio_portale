use leptos::prelude::*;

use crate::maps::map::LayerContext;

#[component]
pub fn LayerControl() -> impl IntoView {
    let layer_ctx = expect_context::<LayerContext>();

    view! {
        <div class="absolute bottom-4 right-4 z-[1000] flex flex-col gap-1 rounded-md border border-border bg-background shadow-md p-1">
            {move || {
                let defs = layer_ctx.layer_defs.with_value(|d| d.clone());
                let active = layer_ctx.active_layer.get();
                defs.into_iter().map(|def| {
                    let is_active = active == def.name;
                    let layer_name = def.name.clone();
                    let active_cls = if is_active {
                        "bg-primary text-primary-foreground"
                    } else {
                        "hover:bg-accent text-foreground"
                    };
                    view! {
                        <button
                            type="button"
                            on:click=move |_| layer_ctx.active_layer.set(layer_name.clone())
                            class=format!(
                                "px-3 py-1 text-xs font-medium rounded-sm transition-colors {}",
                                active_cls,
                            )
                        >
                            {def.name}
                        </button>
                    }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}
