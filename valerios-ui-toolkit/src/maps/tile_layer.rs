use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::maps::leaflet_bindings::TileLayer;
use crate::maps::map::{LayerContext, MapContext};

fn default_url() -> &'static str {
    "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
}

fn default_attribution() -> &'static str {
    "&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors"
}

#[component]
pub fn TileLayer(
    #[prop(default = default_url())] url: &'static str,
    #[prop(default = default_attribution())] attribution: &'static str,
    #[prop(default = "default")] name: &'static str,
    #[prop(optional)] is_default: Option<bool>,
) -> impl IntoView {
    let map_ctx = expect_context::<MapContext>();
    let layer_ctx = expect_context::<LayerContext>();
    let leaflet_layer: StoredValue<Option<TileLayer>> = StoredValue::new(None);

    let is_default = is_default.unwrap_or(false);

    // Register this layer definition
    let final_name = if name.is_empty() || name == "default" {
        let existing = layer_ctx.layer_defs.with_value(|d| d.len());
        format!("Layer {}", existing + 1)
    } else {
        name.to_string()
    };

    layer_ctx.layer_defs.update_value(|defs| {
        defs.push(crate::maps::map::LayerDef {
            name: final_name.clone(),
            url: url.to_string(),
            attribution: attribution.to_string(),
        });
    });

    // Set as default if no active layer set yet
    if is_default && layer_ctx.active_layer.get_untracked().is_empty() {
        layer_ctx.active_layer.set(final_name.clone());
    }

    // If active layer is empty and no default, set this as active
    if layer_ctx.active_layer.get_untracked().is_empty() {
        layer_ctx.active_layer.set(final_name.clone());
    }

    let my_name = final_name.clone();

    Effect::new(move |_| {
        let is_ready = map_ctx.ready.get();
        let is_active = layer_ctx.active_layer.get() == my_name;

        if !is_ready {
            return;
        }

        if is_active {
            if leaflet_layer.with_value(|l| l.is_none()) {
                let opts = js_sys::Object::new();
                js_sys::Reflect::set(
                    &opts,
                    &JsValue::from_str("attribution"),
                    &JsValue::from_str(attribution),
                )
                .ok();
                js_sys::Reflect::set(
                    &opts,
                    &JsValue::from_str("maxZoom"),
                    &JsValue::from_f64(19.0),
                )
                .ok();

                let tl = crate::maps::leaflet_bindings::tileLayer(
                    url,
                    &JsValue::from(&opts),
                );

                map_ctx.map.with_value(|m| {
                    if let Some(ref map) = *m {
                        tl.addTo(map);
                    }
                });

                leaflet_layer.set_value(Some(tl));
            }
        } else {
            let mut removed = None;
            leaflet_layer.update_value(|l| {
                removed = l.take();
            });
            if let Some(tl) = removed {
                tl.remove();
            }
        }
    });

    on_cleanup(move || {
        let mut removed = None;
        leaflet_layer.update_value(|l| {
            removed = l.take();
        });
        if let Some(tl) = removed {
            tl.remove();
        }
    });

    ()
}
