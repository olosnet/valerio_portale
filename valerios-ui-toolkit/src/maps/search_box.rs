use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

fn default_geocoding_url() -> &'static str {
    "https://nominatim.openstreetmap.org/search?format=json&limit=5&q={query}"
}

fn default_elevation_url() -> &'static str {
    "https://api.open-meteo.com/v1/elevation?latitude={lat}&longitude={lng}"
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub lat: f64,
    pub lng: f64,
    pub altitude: Option<f64>,
    pub display_name: String,
}

#[derive(Clone, Debug)]
struct SearchSuggestion {
    lat: f64,
    lng: f64,
    display_name: String,
}

async fn fetch_json(url: &str) -> Result<JsValue, String> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;

    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?;
    Ok(json)
}

fn parse_nominatim(json: &JsValue) -> Vec<SearchSuggestion> {
    let array = js_sys::Array::from(json);
    let mut results = Vec::new();
    for item in array.iter() {
        let lat_str = js_sys::Reflect::get(&item, &JsValue::from_str("lat"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        let lng_str = js_sys::Reflect::get(&item, &JsValue::from_str("lon"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        let display_name = js_sys::Reflect::get(&item, &JsValue::from_str("display_name"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        let lat: f64 = lat_str.parse().unwrap_or(0.0);
        let lng: f64 = lng_str.parse().unwrap_or(0.0);
        if lat != 0.0 || lng != 0.0 {
            results.push(SearchSuggestion {
                lat,
                lng,
                display_name,
            });
        }
    }
    results
}

async fn fetch_altitude(lat: f64, lng: f64, url_template: &str) -> Result<f64, String> {
    let url = url_template
        .replace("{lat}", &lat.to_string())
        .replace("{lng}", &lng.to_string());
    let json = fetch_json(&url).await?;
    let elev_arr = js_sys::Reflect::get(&json, &JsValue::from_str("elevation"))
        .map_err(|e| format!("{e:?}"))?;
    let first = js_sys::Array::from(&elev_arr).get(0);
    Ok(first.as_f64().unwrap_or(0.0))
}

#[component]
pub fn SearchBox(
    #[prop(default = true)] enabled: bool,
    #[prop(default = default_geocoding_url())] geocoding_url: &'static str,
    #[prop(default = false)] fetch_elevation: bool,
    #[prop(default = default_elevation_url())] elevation_url_base: &'static str,
    #[prop(optional)] on_select: Option<Callback<SearchResult>>,
) -> impl IntoView {
    if !enabled {
        return ().into_any();
    }

    let query: RwSignal<String> = RwSignal::new(String::new());
    let suggestions: RwSignal<Vec<SearchSuggestion>> = RwSignal::new(Vec::new());
    let show_dropdown: RwSignal<bool> = RwSignal::new(false);
    let loading: RwSignal<bool> = RwSignal::new(false);
    let debounce_handle: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
    let window = web_sys::window().unwrap();

    let on_input = {
        let query = query.clone();
        let suggestions = suggestions.clone();
        let show_dropdown = show_dropdown.clone();
        let loading = loading.clone();
        let debounce_handle = debounce_handle.clone();
        let geocoding_url = geocoding_url.to_string();
        let window = window.clone();

        move |e: leptos::ev::Event| {
            let val = event_target_value(&e);
            query.set(val.clone());

            if let Some(handle) = debounce_handle.borrow_mut().take() {
                let _ = window.clear_timeout_with_handle(handle);
            }

            if val.trim().is_empty() {
                suggestions.set(Vec::new());
                show_dropdown.set(false);
                return;
            }

            loading.set(true);

            let query = query.clone();
            let suggestions = suggestions.clone();
            let show_dropdown = show_dropdown.clone();
            let loading = loading.clone();
            let geocoding_url = geocoding_url.clone();

            let timeout = Closure::once(move || {
                let encoded = js_sys::encode_uri_component(&query.get_untracked())
                    .as_string()
                    .unwrap_or_default();
                let url = geocoding_url.replace("{query}", &encoded);
                wasm_bindgen_futures::spawn_local(async move {
                    match fetch_json(&url).await {
                        Ok(json) => {
                            let parsed = parse_nominatim(&json);
                            suggestions.set(parsed);
                            show_dropdown.set(true);
                        }
                        Err(_) => {
                            suggestions.set(Vec::new());
                            show_dropdown.set(false);
                        }
                    }
                    loading.set(false);
                });
            });
            let window = web_sys::window().unwrap();
            let handle = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    timeout.as_ref().unchecked_ref(),
                    400,
                )
                .unwrap();
            timeout.forget();
            *debounce_handle.borrow_mut() = Some(handle);
        }
    };

    let on_key = {
        let query = query.clone();
        let loading = loading.clone();
        let suggestions = suggestions.clone();
        let show_dropdown = show_dropdown.clone();
        let geocoding_url = geocoding_url.to_string();
        let window = window.clone();

        move |e: leptos::ev::KeyboardEvent| {
            if e.key() == "Enter" {
                let val = query.get();
                if val.trim().is_empty() {
                    return;
                }
                // Clear debounce
                if let Some(handle) = debounce_handle.borrow_mut().take() {
                    let _ = window.clear_timeout_with_handle(handle);
                }
                loading.set(true);
                let query = query.clone();
                let suggestions = suggestions.clone();
                let show_dropdown = show_dropdown.clone();
                let loading = loading.clone();
                let geocoding_url = geocoding_url.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let encoded = js_sys::encode_uri_component(&query.get_untracked())
                        .as_string()
                        .unwrap_or_default();
                    let url = geocoding_url.replace("{query}", &encoded);
                    match fetch_json(&url).await {
                        Ok(json) => {
                            let parsed = parse_nominatim(&json);
                            suggestions.set(parsed);
                            show_dropdown.set(true);
                        }
                        Err(_) => {
                            suggestions.set(Vec::new());
                            show_dropdown.set(false);
                        }
                    }
                    loading.set(false);
                });
            }
        }
    };

    let on_select_result = {
        let on_select = on_select.clone();
        let loading = loading.clone();
        let show_dropdown = show_dropdown.clone();
        let query = query.clone();
        let suggestions = suggestions.clone();
        let fetch_elevation = fetch_elevation;
        let elevation_url_base = elevation_url_base.to_string();

        move |suggestion: SearchSuggestion| {
            loading.set(true);
            show_dropdown.set(false);
            suggestions.set(Vec::new());
            query.set(String::new());

            let on_select = on_select.clone();
            let elevation_url_base = elevation_url_base.clone();
            let lat = suggestion.lat;
            let lng = suggestion.lng;
            let display_name = suggestion.display_name.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let altitude = if fetch_elevation {
                    fetch_altitude(lat, lng, &elevation_url_base).await.ok()
                } else {
                    None
                };

                if let Some(ref cb) = on_select {
                    cb.run(SearchResult {
                        lat,
                        lng,
                        altitude,
                        display_name,
                    });
                }

                loading.set(false);
            });
        }
    };

    let svg_search = view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16" height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="shrink-0 text-muted-foreground"
        >
            <circle cx="11" cy="11" r="8"/>
            <path d="m21 21-4.35-4.35"/>
        </svg>
    };

    view! {
        <div class="relative">
            <div class="flex items-center gap-2 px-3 py-2 rounded-md border border-input bg-background text-sm">
                {svg_search}
                <input
                    type="text"
                    placeholder="Cerca un luogo..."
                    prop:value=query
                    on:input=on_input
                    on:keypress=on_key
                    class="flex-1 bg-transparent text-foreground placeholder:text-muted-foreground outline-none border-none p-0"
                />
                {move || if loading.get() {
                    view! {
                        <svg class="animate-spin size-4 text-muted-foreground" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            {move || {
                if show_dropdown.get() && !suggestions.get().is_empty() {
                    let items = suggestions.get();
                    let on_select = on_select_result.clone();
                    view! {
                        <div class="absolute top-full left-0 right-0 mt-1 border border-border rounded-md bg-background shadow-md max-h-48 overflow-y-auto z-[1001]">
                            {items.into_iter().map(move |s| {
                                let name = s.display_name.clone();
                                let on_click = on_select.clone();
                                let s_clone = s.clone();
                                view! {
                                    <button
                                        type="button"
                                        on:click=move |_| on_click(s_clone.clone())
                                        class="w-full text-left px-3 py-2 text-sm text-foreground hover:bg-accent transition-colors border-b border-border last:border-b-0 truncate"
                                    >
                                        {name}
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                } else { ().into_any() }
            }}
        </div>
    }.into_any()
}
