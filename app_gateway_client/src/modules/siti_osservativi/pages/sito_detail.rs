use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map};
use serde::Deserialize;

use crate::modules::base::toast_utils::{toast_error, toast_success, use_toast_ctx};
use crate::stores::auth_store::use_auth;

use valerios_ui_toolkit::button::{Button, ButtonVariant};
use valerios_ui_toolkit::confirm_delete::ConfirmDeleteDialog;
use valerios_ui_toolkit::icon::Icon;
use valerios_ui_toolkit::maps::{ElevationButton, LayerControl, Map, Marker, SearchBox, TileLayer};

#[derive(Deserialize, Clone)]
struct EnumValue {
    name: String,
    value: String,
}

#[derive(Deserialize)]
struct StaticsResponse {
    timezones: Vec<EnumValue>,
}

#[component]
pub fn SitoDetail() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let toast = use_toast_ctx();
    let client = auth.api_client.clone();
    let params = use_params_map();
    let get_id = move || params.get().get("id").map(|s| s.to_string());

    let sito = RwSignal::new(None::<app_modules::astronomia::siti_osservativi::models::SitoOsservativo>);

    let nome = RwSignal::new(String::new());
    let lat = RwSignal::new(0.0);
    let lng = RwSignal::new(0.0);
    let alt = RwSignal::new(0.0);
    let timezone = RwSignal::new(Some(String::new()));
    let all_timezones: RwSignal<Vec<EnumValue>> = RwSignal::new(Vec::new());
    let tz_search: RwSignal<String> = RwSignal::new(String::new());
    let show_tz_dropdown: RwSignal<bool> = RwSignal::new(false);

    let delete_open = RwSignal::new(false);

    {
        let client = client.clone();
        spawn_local(async move {
            match client.request("GET", "/statics", None).await {
                Ok(json) => {
                    if let Ok(resp) = serde_json::from_str::<StaticsResponse>(&json) {
                        all_timezones.set(resp.timezones);
                    }
                }
                Err(_) => {}
            }
        });
    }

    {
        let client = client.clone();
        let id = get_id();
        spawn_local(async move {
            if let Some(ref id_val) = id {
                match crate::modules::siti_osservativi::api::get_sito(&client, id_val).await {
                    Ok(s) => {
                        nome.set(s.nome.clone());
                        lat.set(s.latitudine);
                        lng.set(s.longitudine);
                        alt.set(s.altitudine);
                        let tz = s.timezone.clone().unwrap_or_default();
                        timezone.set(Some(tz.clone()));
                        tz_search.set(tz);
                        sito.set(Some(s));
                    }
                    Err(e) => toast_error(&toast, &e.to_string()),
                }
            }
        });
    }

    let on_delete = {
        let client = client.clone();
        let toast = toast.clone();
        let navigate = navigate.clone();
        let get_id = get_id.clone();
        Callback::new(move |_| {
            let id = get_id();
            let navigate = navigate.clone();
            let toast = toast.clone();
            spawn_local({
                let client = client.clone();
                async move {
                    if let Some(ref id_val) = id {
                        match crate::modules::siti_osservativi::api::delete_sito(&client, id_val)
                            .await
                        {
                            Ok(()) => {
                                toast_success(&toast, "Sito osservativo eliminato");
                                let _ = navigate("/siti_osservativi", Default::default());
                            }
                            Err(e) => toast_error(&toast, &e.to_string()),
                        }
                    }
                }
            });
        })
    };

    let on_search = Callback::new(move |r: valerios_ui_toolkit::maps::SearchResult| {
        lat.set(r.lat);
        lng.set(r.lng);
        if let Some(a) = r.altitude {
            alt.set(a);
        }
    });

    let on_save = {
        let client = client.clone();
        let get_id = get_id.clone();
        let toast = toast.clone();
        let sito = sito.clone();
        move |_| {
            let id = get_id();
            let body = app_modules::astronomia::siti_osservativi::models::SitoOsservativoUpdate {
                nome: nome.get(),
                longitudine: lng.get(),
                latitudine: lat.get(),
                altitudine: alt.get(),
                timezone: timezone.get(),
            };
            let toast = toast.clone();
            spawn_local({
                let client = client.clone();
                async move {
                    if let Some(ref id_val) = id {
                        match crate::modules::siti_osservativi::api::update_sito(
                            &client, id_val, &body,
                        )
                        .await
                        {
                            Ok(s) => {
                                sito.set(Some(s));
                                toast_success(&toast, "Sito osservativo aggiornato");
                            }
                            Err(e) => toast_error(&toast, &e.to_string()),
                        }
                    }
                }
            });
        }
    };

    view! {
        <Title text="Dettaglio sito osservativo - App Gateway"/>

        <div class="mx-auto space-y-8">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <a
                        href="/siti_osservativi"
                        class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground h-8 w-8 border border-input bg-background"
                    >
                        {Icon::ArrowLeft.render()}
                    </a>
                    <div>
                        <h2 class="text-xl font-semibold text-foreground mb-1">
                            {move || nome.get()}
                        </h2>
                        <p class="text-sm text-muted-foreground">
                            "Dettaglio e modifica sito osservativo"
                        </p>
                    </div>
                </div>
                <Button
                    variant=ButtonVariant::Destructive
                    size=valerios_ui_toolkit::button::ButtonSize::Icon
                    on_click=Arc::new(move || delete_open.set(true))
                >
                    {Icon::Trash.render()}
                </Button>
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6">
                <h3 class="text-lg font-medium text-foreground mb-4">"Dati sito"</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Nome"</label>
                            <input
                                type="text"
                                prop:value=nome
                                on:input=move |e| nome.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Latitudine"</label>
                            <input
                                type="number" step="any"
                                prop:value=move || format!("{:.6}", lat.get())
                                on:input=move |e| {
                                    lat.set(event_target_value(&e).parse().unwrap_or(0.0));
                                }
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Longitudine"</label>
                            <input
                                type="number" step="any"
                                prop:value=move || format!("{:.6}", lng.get())
                                on:input=move |e| {
                                    lng.set(event_target_value(&e).parse().unwrap_or(0.0));
                                }
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Altitudine (m)"</label>
                            <div class="flex items-center gap-2">
                                <input
                                    type="number" step="any"
                                    prop:value=move || format!("{:.1}", alt.get())
                                    on:input=move |e| {
                                        alt.set(event_target_value(&e).parse().unwrap_or(0.0));
                                    }
                                    class="flex-1 px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                                />
                                <ElevationButton lat=lat lng=lng on_altitude=Callback::new(move |a| alt.set(a)) />
                            </div>
                        </div>

                        <div class="relative">
                            <label class="block text-sm font-medium text-foreground mb-1">"Timezone"</label>
                            <input
                                type="text"
                                prop:value=move || tz_search.get()
                                placeholder="Cerca fuso orario..."
                                on:focus=move |_| show_tz_dropdown.set(true)
                                on:input=move |e| {
                                    tz_search.set(event_target_value(&e));
                                    show_tz_dropdown.set(true);
                                }
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                            />
                            {move || {
                                if !show_tz_dropdown.get() { return ().into_any(); }
                                let q = tz_search.get().to_lowercase();
                                let filtered: Vec<EnumValue> = all_timezones.with(|tzs| {
                                    tzs.iter()
                                        .filter(|t| q.is_empty() || t.name.to_lowercase().contains(&q) || t.value.to_lowercase().contains(&q))
                                        .take(20)
                                        .cloned()
                                        .collect()
                                });
                                if filtered.is_empty() { return ().into_any(); }
                                let btn_items: Vec<AnyView> = filtered.into_iter().map(move |tz| {
                                    let name = tz.name.clone();
                                    let val = tz.value;
                                    let disp = tz.name;
                                    view! {
                                        <button
                                            type="button"
                                            on:click=move |_| {
                                                timezone.set(Some(val.clone()));
                                                tz_search.set(name.clone());
                                                show_tz_dropdown.set(false);
                                            }
                                            class="w-full text-left px-3 py-2 text-sm text-foreground hover:bg-accent transition-colors border-b border-border last:border-b-0 truncate"
                                        >
                                            {disp}
                                        </button>
                                    }.into_any()
                                }).collect();
                                view! {
                                    <div class="absolute top-full left-0 right-0 mt-1 border border-border rounded-md bg-background shadow-md max-h-48 overflow-y-auto z-[1002]">
                                        {btn_items}
                                    </div>
                                }.into_any()
                            }}
                            {move || if show_tz_dropdown.get() {
                                view! {
                                    <div on:click=move |_| show_tz_dropdown.set(false) class="fixed inset-0 z-[1001]" />
                                }.into_any()
                            } else { ().into_any() }}
                        </div>
                    </div>

                    <div class="space-y-4">
                        <SearchBox enabled=true fetch_elevation=true on_select=on_search />
                        <div class="relative">
                            <Map
                                center_lat=41.9
                                center_lng=12.5
                                zoom=5.0
                                height="400px"
                            >
                                <TileLayer
                                    name="OpenStreetMap"
                                    is_default=true
                                />
                                <TileLayer
                                    name="Satellite"
                                    url="https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}"
                                    attribution="&copy; Esri"
                                />
                                <LayerControl />
                                <Marker
                                    lat=lat
                                    lng=lng
                                    draggable=true
                                    popup_content=format!("{:.4}, {:.4}", lat.get_untracked(), lng.get_untracked())
                                />
                            </Map>
                        </div>
                    </div>
                </div>

                <hr class="my-6 border-border" />

                <div class="flex justify-end">
                    <button
                        on:click=on_save
                        class="px-6 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors"
                    >
                        "Salva modifiche"
                    </button>
                </div>
            </div>

            <ConfirmDeleteDialog
                open=delete_open
                item_type="sito osservativo"
                on_confirm=on_delete
            />
        </div>
    }
}
