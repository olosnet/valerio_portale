use js_sys::Array;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::maps::leaflet_bindings::*;
use crate::maps::map::MapContext;

#[component]
pub fn Marker(
    lat: RwSignal<f64>,
    lng: RwSignal<f64>,
    #[prop(optional)] draggable: Option<bool>,
    #[prop(optional)] popup_content: Option<String>,
) -> impl IntoView {
    let map_ctx = expect_context::<MapContext>();
    let leaflet_marker: StoredValue<Option<Marker>> = StoredValue::new(None);
    let is_draggable = draggable.unwrap_or(false);
    let prev_pos: StoredValue<Option<(f64, f64)>> = StoredValue::new(None);

    {
        let leaflet_marker = leaflet_marker.clone();
        let map_ctx = map_ctx.clone();
        Effect::new(move |_| {
            let is_ready = map_ctx.ready.get();
            if !is_ready {
                return;
            }
            if leaflet_marker.with_value(|m| m.is_some()) {
                return;
            }

            let lat_val = lat.get();
            let lng_val = lng.get();

            let latlng = Array::new();
            latlng.push(&JsValue::from_f64(lat_val));
            latlng.push(&JsValue::from_f64(lng_val));

            let opts = js_sys::Object::new();
            js_sys::Reflect::set(
                &opts,
                &JsValue::from_str("draggable"),
                &JsValue::from_bool(is_draggable),
            )
            .ok();

            let m = crate::maps::leaflet_bindings::marker(
                &latlng,
                &JsValue::from(&opts),
            );

            if let Some(ref content) = popup_content {
                m.bindPopup(content);
            }

            if is_draggable {
                let drag_cb = {
                    let lat = lat.clone();
                    let lng = lng.clone();
                    let prev_pos = prev_pos.clone();
                    Closure::new(move |e: JsValue| {
                        if let Ok(target) = js_sys::Reflect::get(
                            &e,
                            &JsValue::from_str("target"),
                        ) {
                            let marker_obj: Marker = target.unchecked_into();
                            let latlng = marker_obj.getLatLng();
                            let new_lat = latlng.lat();
                            let new_lng = latlng.lng();
                            lat.set(new_lat);
                            lng.set(new_lng);
                            prev_pos.set_value(Some((new_lat, new_lng)));
                        }
                    })
                };
                m.on("dragend", &drag_cb);
                drag_cb.forget();
            }

            map_ctx.map.with_value(|opt_map| {
                if let Some(ref map) = *opt_map {
                    m.addTo(map);
                }
            });

            leaflet_marker.set_value(Some(m));
        });
    }

    Effect::new(move |_| {
        let lat_val = lat.get();
        let lng_val = lng.get();

        if prev_pos.with_value(|p| *p == Some((lat_val, lng_val))) {
            return;
        }
        prev_pos.set_value(Some((lat_val, lng_val)));

        leaflet_marker.with_value(|m| {
            if let Some(ref marker) = *m {
                let latlng = Array::new();
                latlng.push(&JsValue::from_f64(lat_val));
                latlng.push(&JsValue::from_f64(lng_val));
                marker.setLatLng(&latlng);
            }
        });
    });

    on_cleanup(move || {
        leaflet_marker.update_value(|m| *m = None);
    });

    ()
}
