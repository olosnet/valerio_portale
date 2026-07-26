use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_namespace = L)]
extern "C" {
    pub fn map(element: &JsValue, options: &JsValue) -> Map;
    pub fn tileLayer(url: &str, options: &JsValue) -> TileLayer;
    pub fn marker(latlng: &Array, options: &JsValue) -> Marker;
}

#[wasm_bindgen]
extern "C" {
    pub type Map;

    #[wasm_bindgen(method)]
    pub fn setView(this: &Map, center: &Array, zoom: f64) -> Map;

    #[wasm_bindgen(method, js_name = on)]
    pub fn on(this: &Map, event_type: &str, callback: &Closure<dyn Fn(JsValue)>) -> Map;

    #[wasm_bindgen(method)]
    pub fn remove(this: &Map);

    pub type TileLayer;

    #[wasm_bindgen(method, js_name = addTo)]
    pub fn addTo(this: &TileLayer, map: &Map) -> TileLayer;

    #[wasm_bindgen(method)]
    pub fn remove(this: &TileLayer);

    pub type Marker;

    #[wasm_bindgen(method, js_name = addTo)]
    pub fn addTo(this: &Marker, map: &Map) -> Marker;

    #[wasm_bindgen(method, js_name = on)]
    pub fn on(this: &Marker, event_type: &str, callback: &Closure<dyn Fn(JsValue)>) -> Marker;

    #[wasm_bindgen(method)]
    pub fn bindPopup(this: &Marker, content: &str) -> Marker;

    #[wasm_bindgen(method)]
    pub fn getLatLng(this: &Marker) -> LatLng;

    #[wasm_bindgen(method)]
    pub fn setLatLng(this: &Marker, latlng: &Array);

    pub type LatLng;

    #[wasm_bindgen(method, getter)]
    pub fn lat(this: &LatLng) -> f64;

    #[wasm_bindgen(method, getter)]
    pub fn lng(this: &LatLng) -> f64;

    pub type DragEndEvent;

    #[wasm_bindgen(method, getter, js_name = target)]
    pub fn target(this: &DragEndEvent) -> Marker;

    pub type LeafletMouseEvent;

    #[wasm_bindgen(method, getter)]
    pub fn latlng(this: &LeafletMouseEvent) -> LatLng;
}
