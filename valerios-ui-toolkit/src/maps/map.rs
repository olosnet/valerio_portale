use js_sys::Array;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::maps::leaflet_bindings::*;

#[derive(Clone)]
pub struct MapContext {
    pub map: StoredValue<Option<Map>>,
    pub ready: RwSignal<bool>,
}

#[derive(Clone)]
pub struct LayerDef {
    pub name: String,
    pub url: String,
    pub attribution: String,
}

#[derive(Clone)]
pub struct LayerContext {
    pub active_layer: RwSignal<String>,
    pub layer_defs: StoredValue<Vec<LayerDef>>,
}

#[component]
pub fn Map(
    children: ChildrenFn,
    center_lat: f64,
    center_lng: f64,
    zoom: f64,
    #[prop(optional)] on_click: Option<Callback<(f64, f64)>>,
    #[prop(default = false)] clickable: bool,
    #[prop(default = "100%")] width: &'static str,
    #[prop(default = "500px")] height: &'static str,
    #[prop(default = "")] default_layer: &'static str,
) -> impl IntoView {
    let map_node: NodeRef<leptos::html::Div> = NodeRef::new();
    let ctx = MapContext {
        map: StoredValue::new(None),
        ready: RwSignal::new(false),
    };
    provide_context(ctx.clone());

    let layer_ctx = LayerContext {
        active_layer: RwSignal::new(default_layer.to_string()),
        layer_defs: StoredValue::new(Vec::new()),
    };
    provide_context(layer_ctx.clone());

    {
        let ctx_map = ctx.map.clone();
        let on_click = on_click.clone();
        let ctx_ready = ctx.ready.clone();
        Effect::new(move |_| {
            if ctx_map.with_value(|m| m.is_some()) {
                return;
            }
            let el = map_node.get();
            if el.is_none() {
                return;
            }
            set_leaflet_icon_path();

            let opts = js_sys::Object::new();
            js_sys::Reflect::set(
                &opts,
                &JsValue::from_str("zoomControl"),
                &JsValue::from_bool(true),
            )
            .ok();
            js_sys::Reflect::set(
                &opts,
                &JsValue::from_str("attributionControl"),
                &JsValue::from_bool(true),
            )
            .ok();
            js_sys::Reflect::set(
                &opts,
                &JsValue::from_str("scrollWheelZoom"),
                &JsValue::from_bool(true),
            )
            .ok();

            let m = map(&JsValue::from(&*el.unwrap()), &JsValue::from(&opts));

            let center = Array::new();
            center.push(&JsValue::from_f64(center_lat));
            center.push(&JsValue::from_f64(center_lng));
            m.setView(&center, zoom);

            if clickable { if let Some(cb) = on_click.clone() {
                let click_cb = Closure::new(move |e: JsValue| {
                    if let Ok(latlng_val) =
                        js_sys::Reflect::get(&e, &JsValue::from_str("latlng"))
                    {
                        let lat = js_sys::Reflect::get(
                            &latlng_val,
                            &JsValue::from_str("lat"),
                        )
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                        let lng = js_sys::Reflect::get(
                            &latlng_val,
                            &JsValue::from_str("lng"),
                        )
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                        cb.run((lat, lng));
                    }
                });
                m.on("click", &click_cb);
                click_cb.forget();
            } }

            ctx_map.set_value(Some(m));
            ctx_ready.set(true);
        });
    }

    on_cleanup(move || {
        let mut removed = None;
        ctx.map.update_value(|m| {
            removed = m.take();
        });
        if let Some(m) = removed {
            m.remove();
        }
    });

    view! {
        <div
            node_ref=map_node
            style=format!("width:{};height:{};z-index:0", width, height)
        >
            {children()}
        </div>
    }
}

fn set_leaflet_icon_path() {
    if let Ok(l) = js_sys::Reflect::get(
        &js_sys::global(),
        &JsValue::from_str("L"),
    ) {
        if let Ok(icon) =
            js_sys::Reflect::get(&l, &JsValue::from_str("Icon"))
        {
            if let Ok(default_icon) = js_sys::Reflect::get(
                &icon,
                &JsValue::from_str("Default"),
            ) {
                js_sys::Reflect::set(
                    &default_icon,
                    &JsValue::from_str("imagePath"),
                    &JsValue::from_str("/images/"),
                )
                .ok();
            }
        }
    }
}
