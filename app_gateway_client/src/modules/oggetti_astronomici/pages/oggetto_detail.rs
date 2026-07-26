use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map};
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::FileReader;
use js_sys::Uint8Array;

use crate::modules::base::toast_utils::{toast_error, toast_success, use_toast_ctx};
use crate::stores::auth_store::use_auth;

use app_modules::astronomia::oggetti_astronomici::models::{
    CatalogoInput, Costellazione, OggettoAstronomico, OggettoAstronomicoUpdate, TipoOggetto,
};
use app_modules::astronomia::common::helpers::{format_ar, format_dec};
use valerios_ui_toolkit::button::{Button, ButtonVariant};
use valerios_ui_toolkit::button::ButtonSize;
use valerios_ui_toolkit::confirm_delete::ConfirmDeleteDialog;
use valerios_ui_toolkit::icon::Icon;

#[derive(Deserialize, Clone)]
struct EnumValue { name: String, value: String }

#[derive(Deserialize)]
struct StaticsResp { tipo_oggetto: Vec<EnumValue>, costellazioni: Vec<EnumValue> }

fn parse_tipo(s: &str) -> TipoOggetto {
    serde_json::from_str(&format!("\"{}\"", s)).unwrap_or_default()
}

const CATALOG_IDS: &[&str] = &["M", "NGC", "IC", "C", "HIP", "HD", "B", "Col", "Mel", "UGC", "PGC", "vdB", "Abell", "Sh2", "Orl"];

#[component]
pub fn OggettoDetail() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let toast = use_toast_ctx();
    let client = auth.api_client.clone();
    let params = use_params_map();
    let get_id = move || params.get().get("id").map(|s| s.to_string());

    let form_nome = RwSignal::new(String::new());
    let form_tipo_val = RwSignal::new("GAL".to_string());
    let form_cost_val = RwSignal::new("Sconosciuta".to_string());
    let form_ar = RwSignal::new(String::new());
    let form_dec = RwSignal::new(String::new());
    let form_mag = RwSignal::new(String::new());
    let form_note = RwSignal::new(String::new());

    let form_image = RwSignal::new(None::<String>);
    let cataloghi = RwSignal::new(Vec::<(String, String)>::new());

    let delete_open = RwSignal::new(false);
    let uploading = RwSignal::new(false);
    let pending_upload = RwSignal::new(None::<(Vec<u8>, String, String)>);

    let oggetto = RwSignal::new(None::<OggettoAstronomico>);

    let all_tipi: RwSignal<Vec<EnumValue>> = RwSignal::new(Vec::new());
    let all_costellazioni: RwSignal<Vec<EnumValue>> = RwSignal::new(Vec::new());
    let tipo_search = RwSignal::new(String::new());
    let cost_search = RwSignal::new(String::new());
    let show_tipo_dropdown = RwSignal::new(false);
    let show_cost_dropdown = RwSignal::new(false);

    {
        let client = client.clone();
        spawn_local(async move {
            if let Ok(json) = client.request("GET", "/statics", None).await {
                if let Ok(resp) = serde_json::from_str::<StaticsResp>(&json) {
                    all_tipi.set(resp.tipo_oggetto.clone());
                    all_costellazioni.set(resp.costellazioni.clone());
                    if !form_tipo_val.get().is_empty() {
                        if let Some(ev) = resp.tipo_oggetto.iter().find(|t| t.value == form_tipo_val.get()) {
                            tipo_search.set(ev.name.clone());
                        }
                    }
                    if !form_cost_val.get().is_empty() {
                        if let Some(ev) = resp.costellazioni.iter().find(|c| c.value == form_cost_val.get()) {
                            cost_search.set(ev.name.clone());
                        }
                    }
                }
            }
        });
    }

    {
        let client = client.clone();
        let id = get_id();
        spawn_local(async move {
            if let Some(ref id_val) = id {
                match crate::modules::oggetti_astronomici::api::get_oggetto(&client, id_val).await {
                    Ok(o) => {
                        let cost_val = o.abbr_costellazione.to_string();
                        let tipo_val = o.tipo.to_string();
                        form_nome.set(o.nome_comune.clone());
                        form_tipo_val.set(tipo_val.clone());
                        form_cost_val.set(cost_val.clone());
                        tipo_search.set(tipo_val.clone());
                        cost_search.set(cost_val.clone());
                        form_ar.set(o.coord_ar.clone());
                        form_dec.set(o.coord_dec.clone());
                        form_mag.set(o.mag_apparente.map(|m| m.to_string()).unwrap_or_default());
                        form_note.set(o.note.clone());
                        form_image.set(o.image_filename.clone());
                        cataloghi.set(o.cataloghi.iter().map(|c| (c.catalog_id.clone(), c.catalog_nr.clone())).collect());
                        oggetto.set(Some(o));

                        let all_t = all_tipi.get_untracked();
                        let all_c = all_costellazioni.get_untracked();
                        if !all_t.is_empty() {
                            if let Some(ev) = all_t.iter().find(|t| t.value == tipo_val) {
                                tipo_search.set(ev.name.clone());
                            }
                        }
                        if !all_c.is_empty() {
                            if let Some(ev) = all_c.iter().find(|c| c.value == cost_val) {
                                cost_search.set(ev.name.clone());
                            }
                        }
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
                        match crate::modules::oggetti_astronomici::api::delete_oggetto(&client, id_val).await {
                            Ok(()) => { toast_success(&toast, "Oggetto astronomico eliminato"); let _ = navigate("/oggetti_astronomici", Default::default()); }
                            Err(e) => toast_error(&toast, &e.to_string()),
                        }
                    }
                }
            });
        })
    };

    let on_save = {
        let client = client.clone();
        let get_id = get_id.clone();
        let toast = toast.clone();
        let oggetto = oggetto;
        move |_| {
            let id = get_id();
            let body = OggettoAstronomicoUpdate {
                tipo: parse_tipo(&form_tipo_val.get()),
                nome_comune: form_nome.get(),
                abbr_costellazione: Costellazione::parse(&form_cost_val.get()),
                coord_ar: form_ar.get(),
                coord_dec: form_dec.get(),
                mag_apparente: form_mag.get().parse().ok(),
                dim_apparenti: None,
                note: form_note.get(),
                multi: false,
                imported: false,
                cataloghi: cataloghi.get().into_iter().map(|(id, nr)| CatalogoInput { catalog_id: id, catalog_nr: nr }).collect(),
            };
            let toast = toast.clone();
            spawn_local({
                let client = client.clone();
                async move {
                    if let Some(ref id_val) = id {
                        match crate::modules::oggetti_astronomici::api::update_oggetto(&client, id_val, &body).await {
                            Ok(o) => { oggetto.set(Some(o)); toast_success(&toast, "Oggetto astronomico aggiornato"); }
                            Err(e) => toast_error(&toast, &e.to_string()),
                        }
                    }
                }
            });
        }
    };

    let handle_file_select = move |ev: leptos::ev::Event| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);
        if let Some(files) = target.files() {
            if let Some(file) = files.get(0) {
                let mime = file.type_();
                let reader = FileReader::new().unwrap();
                let pu = pending_upload;
                let file_clone = file.clone();
                let onload: Closure<dyn FnMut(web_sys::ProgressEvent)> =
                    Closure::new(Box::new(move |pe: web_sys::ProgressEvent| {
                        let reader: FileReader = pe.target().unwrap().dyn_into().unwrap();
                        if let Ok(result) = reader.result() {
                            let array = Uint8Array::new(&result);
                            let mut bytes = vec![0u8; array.length() as usize];
                            array.copy_to(&mut bytes);
                            let mt = mime.clone();
                            let fname = file_clone.name();
                            pu.set(Some((bytes, fname, mt)));
                        }
                    }));
                let _ = reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();
                let _ = reader.read_as_array_buffer(&file);
            }
            let _ = target.set_value("");
        }
    };

    let image_url = Signal::derive(move || form_image.get().map(|f| format!("/api/filemanager/{}", f)));
    let ar_preview = Signal::derive(move || format_ar(&form_ar.get()));
    let dec_preview = Signal::derive(move || format_dec(&form_dec.get()));

    view! {
        <Title text="Dettaglio oggetto astronomico - App Gateway"/>
        <div class="mx-auto space-y-8">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <a href="/oggetti_astronomici"
                        class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground h-8 w-8 border border-input bg-background"
                    >{Icon::ArrowLeft.render()}</a>
                    <div>
                        <h2 class="text-xl font-semibold text-foreground mb-1">{move || form_nome.get()}</h2>
                        <p class="text-sm text-muted-foreground">"Dettaglio e modifica oggetto astronomico"</p>
                    </div>
                </div>
                <Button variant=ButtonVariant::Destructive size=ButtonSize::Icon
                    on_click=Arc::new(move || delete_open.set(true))>{Icon::Trash.render()}</Button>
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6">
                <h3 class="text-lg font-medium text-foreground mb-4">"Dati oggetto"</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Nome comune"</label>
                            <input type="text" prop:value=form_nome
                                on:input=move |e| form_nome.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Tipo"</label>
                            <div class="relative">
                                <input type="text" placeholder="Cerca tipo..."
                                    prop:value=move || tipo_search.get()
                                    on:focus=move |_| show_tipo_dropdown.set(true)
                                    on:input=move |e| { tipo_search.set(event_target_value(&e)); show_tipo_dropdown.set(true); }
                                    class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                                />
                                {move || {
                                    if !show_tipo_dropdown.get() { return ().into_any(); }
                                    let q = tipo_search.get().to_lowercase();
                                    let filtered: Vec<EnumValue> = all_tipi.with(|tipi| {
                                        tipi.iter()
                                            .filter(|t| q.is_empty() || t.name.to_lowercase().contains(&q) || t.value.to_lowercase().contains(&q))
                                            .take(20).cloned().collect()
                                    });
                                    if filtered.is_empty() { return ().into_any(); }
                                    let btn_items: Vec<AnyView> = filtered.into_iter().map(|t| {
                                        let t_name = t.name.clone();
                                        let t_val = t.value.clone();
                                        let btn_text = format!("{} ({})", t_name, t_val);
                                        view! {
                                            <button type="button"
                                                on:click=move |_| { form_tipo_val.set(t_val.clone()); tipo_search.set(t_name.clone()); show_tipo_dropdown.set(false); }
                                                class="w-full text-left px-3 py-2 text-sm text-foreground hover:bg-accent transition-colors border-b border-border last:border-b-0 truncate"
                                            >{btn_text}</button>
                                        }.into_any()
                                    }).collect();
                                    view! {
                                        <div class="absolute top-full left-0 right-0 mt-1 border border-border rounded-md bg-background shadow-md max-h-48 overflow-y-auto z-[1002]">
                                            {btn_items}
                                        </div>
                                    }.into_any()
                                }}
                                {move || if show_tipo_dropdown.get() {
                                    view! { <div on:click=move |_| show_tipo_dropdown.set(false) class="fixed inset-0 z-[1001]" /> }.into_any()
                                } else { ().into_any() }}
                            </div>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Costellazione"</label>
                            <div class="relative">
                                <input type="text" placeholder="Cerca costellazione..."
                                    prop:value=move || cost_search.get()
                                    on:focus=move |_| show_cost_dropdown.set(true)
                                    on:input=move |e| { cost_search.set(event_target_value(&e)); show_cost_dropdown.set(true); }
                                    class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                                />
                                {move || {
                                    if !show_cost_dropdown.get() { return ().into_any(); }
                                    let q = cost_search.get().to_lowercase();
                                    let filtered: Vec<EnumValue> = all_costellazioni.with(|costs| {
                                        costs.iter()
                                            .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q) || c.value.to_lowercase().contains(&q))
                                            .take(20).cloned().collect()
                                    });
                                    if filtered.is_empty() { return ().into_any(); }
                                    let btn_items: Vec<AnyView> = filtered.into_iter().map(|c| {
                                        let c_name = c.name.clone();
                                        let c_val = c.value.clone();
                                        let btn_text = format!("{} ({})", c_name, c_val);
                                        view! {
                                            <button type="button"
                                                on:click=move |_| { form_cost_val.set(c_val.clone()); cost_search.set(c_name.clone()); show_cost_dropdown.set(false); }
                                                class="w-full text-left px-3 py-2 text-sm text-foreground hover:bg-accent transition-colors border-b border-border last:border-b-0 truncate"
                                            >{btn_text}</button>
                                        }.into_any()
                                    }).collect();
                                    view! {
                                        <div class="absolute top-full left-0 right-0 mt-1 border border-border rounded-md bg-background shadow-md max-h-48 overflow-y-auto z-[1002]">
                                            {btn_items}
                                        </div>
                                    }.into_any()
                                }}
                                {move || if show_cost_dropdown.get() {
                                    view! { <div on:click=move |_| show_cost_dropdown.set(false) class="fixed inset-0 z-[1001]" /> }.into_any()
                                } else { ().into_any() }}
                            </div>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"AR (J2000)"</label>
                            <input type="text" prop:value=form_ar
                                on:input=move |e| form_ar.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                            <p class="text-xs text-muted-foreground mt-0.5">{move || ar_preview.get()}</p>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"DEC (J2000)"</label>
                            <input type="text" prop:value=form_dec
                                on:input=move |e| form_dec.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                            <p class="text-xs text-muted-foreground mt-0.5">{move || dec_preview.get()}</p>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Mag. apparente"</label>
                            <input type="text" prop:value=form_mag
                                on:input=move |e| form_mag.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Note"</label>
                            <textarea prop:value=form_note
                                on:input=move |e| form_note.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm min-h-[80px]"/>
                        </div>
                    </div>

                    <div class="space-y-6">
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-2">"Immagine"</label>
                            <div class="flex flex-col items-center gap-3 p-4 rounded-md border border-dashed border-border">
                                {move || {
                                    let url = image_url.get();
                                    if let Some(url) = url {
                                        view! {
                                            <img src=url class="max-w-full max-h-[300px] rounded-md object-contain" alt="Oggetto astronomico" />
                                            {form_image.get().map(|f| view! { <p class="text-xs text-muted-foreground truncate max-w-full">{f}</p> }.into_any())}
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="flex flex-col items-center gap-2 text-muted-foreground py-6">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
                                                <span class="text-sm">"Nessuna immagine"</span>
                                            </div>
                                        }.into_any()
                                    }
                                }}
                                <input type="file" accept="image/*" class="hidden" on:change=handle_file_select id="image-upload"/>
                                <label for="image-upload"
                                    class="inline-flex items-center gap-2 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors cursor-pointer"
                                >
                                    {move || if uploading.get() {
                                        view! { <svg class="animate-spin size-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/></svg> }.into_any()
                                    } else { ().into_any() }}
                                    {move || if uploading.get() { "Caricamento..." } else { "Carica immagine" }}
                                </label>
                            </div>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-medium text-foreground">"Cataloghi"</label>
                                <button type="button" on:click=move |_| cataloghi.update(|v| v.push((String::new(), String::new())))
                                    class="text-xs text-primary underline hover:no-underline">"+ Aggiungi catalogo"</button>
                            </div>
                            <div class="space-y-2">
                                {move || cataloghi.with(|list| {
                                    let mut items: Vec<AnyView> = Vec::new();
                                    for (i, (cid, cnr)) in list.iter().enumerate() {
                                        let idx = RwSignal::new(i);
                                        let row_cid = RwSignal::new(cid.clone());
                                        let row_cnr = RwSignal::new(cnr.clone());
                                        let cat_clone = cataloghi;
                                        let remove = move |_| { cat_clone.update(|v| { v.remove(idx.get()); }); };
                                        let update_id = move |ev| { cataloghi.update(|v| v[idx.get()].0 = event_target_value(&ev)); };
                                        let update_nr = move |ev| { cataloghi.update(|v| v[idx.get()].1 = event_target_value(&ev)); };
                                        items.push(view! {
                                            <div class="flex items-center gap-2">
                                                <select prop:value=move || row_cid.get()
                                                    on:change=update_id
                                                    class="flex-1 px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm">
                                                    <option value="">-</option>
                                                    {CATALOG_IDS.iter().map(|id| view! { <option value={id.to_string()}>{id.to_string()}</option> }.into_any()).collect::<Vec<AnyView>>()}
                                                </select>
                                                <input type="text" prop:value=move || row_cnr.get()
                                                    on:input=update_nr
                                                    class="flex-[2] px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                                                <button type="button" on:click=remove
                                                    class="inline-flex items-center justify-center rounded-md text-muted-foreground hover:text-destructive h-8 w-8">{Icon::X.render()}</button>
                                            </div>
                                        }.into_any());
                                    }
                                    items.into_iter().collect_view()
                                })}
                            </div>
                        </div>
                    </div>
                </div>

                <hr class="my-6 border-border" />
                <div class="flex justify-end">
                    <button on:click=on_save
                        class="px-6 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors">"Salva modifiche"</button>
                </div>
            </div>

            <ConfirmDeleteDialog open=delete_open item_type="oggetto astronomico" on_confirm=on_delete />
        </div>
    }
}
