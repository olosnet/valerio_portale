use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::maps::search_box::{default_elevation_url, fetch_altitude};

#[component]
pub fn ElevationButton(
    lat: RwSignal<f64>,
    lng: RwSignal<f64>,
    on_altitude: Callback<f64>,
    #[prop(optional)] elevation_url: Option<&'static str>,
) -> impl IntoView {
    let loading = RwSignal::new(false);
    let url = elevation_url.unwrap_or_else(default_elevation_url);

    let on_click = move |_| {
        if loading.get() {
            return;
        }
        loading.set(true);
        let lat_val = lat.get();
        let lng_val = lng.get();
        let url = url.to_string();
        spawn_local(async move {
            match fetch_altitude(lat_val, lng_val, &url).await {
                Ok(alt) => on_altitude.run(alt),
                Err(_) => {}
            }
            loading.set(false);
        });
    };

    view! {
        <button
            type="button"
            on:click=on_click
            disabled=move || loading.get()
            class="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 h-9 w-9 border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground shrink-0"
            title="Recupera altitudine"
        >
            {move || if loading.get() {
                view! {
                    <svg class="animate-spin size-4" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
                }.into_any()
            } else {
                view! {
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0"><path d="m8 3 4 8 5-5 5 15H2L8 3z"/></svg>
                }.into_any()
            }}
        </button>
    }
}
