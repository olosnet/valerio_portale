use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map};
use wasm_bindgen::prelude::*;
use web_sys::FileReader;
use js_sys::Uint8Array;

use crate::modules::base::toast_utils::{toast_error, toast_success, use_toast_ctx};
use crate::stores::auth_store::use_auth;

use app_modules::astronomia::oggetti_astronomici::models::{
    Costellazione, OggettoAstronomico, OggettoAstronomicoUpdate, TipoOggetto,
};
use valerios_ui_toolkit::button::{Button, ButtonVariant};
use valerios_ui_toolkit::button::ButtonSize;
use valerios_ui_toolkit::confirm_delete::ConfirmDeleteDialog;
use valerios_ui_toolkit::icon::Icon;

#[component]
pub fn OggettoDetail() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let toast = use_toast_ctx();
    let client = auth.api_client.clone();
    let params = use_params_map();
    let get_id = move || params.get().get("id").map(|s| s.to_string());

    let oggetto = RwSignal::new(None::<OggettoAstronomico>);

    let form_nome = RwSignal::new(String::new());
    let form_tipo = RwSignal::new("GAL".to_string());
    let form_costellazione = RwSignal::new("Sconosciuta".to_string());
    let form_ar = RwSignal::new(String::new());
    let form_dec = RwSignal::new(String::new());
    let form_mag = RwSignal::new(String::new());
    let form_note = RwSignal::new(String::new());
    let form_image = RwSignal::new(None::<String>);

    let delete_open = RwSignal::new(false);
    let uploading = RwSignal::new(false);

    let pending_upload = RwSignal::new(None::<(Vec<u8>, String, String)>);

    {
        let client = client.clone();
        let id = get_id();
        spawn_local(async move {
            if let Some(ref id_val) = id {
                match crate::modules::oggetti_astronomici::api::get_oggetto(&client, id_val).await {
                    Ok(o) => {
                        form_nome.set(o.nome_comune.clone());
                        form_tipo.set(format!("{}", o.tipo));
                        form_costellazione.set(o.abbr_costellazione.to_string());
                        form_ar.set(o.coord_ar.clone());
                        form_dec.set(o.coord_dec.clone());
                        form_mag.set(o.mag_apparente.map(|m| m.to_string()).unwrap_or_default());
                        form_note.set(o.note.clone());
                        form_image.set(o.image_filename.clone());
                        oggetto.set(Some(o));
                    }
                    Err(e) => toast_error(&toast, &e.to_string()),
                }
            }
        });
    }

    Effect::new({
        let client = client.clone();
        let get_id = get_id.clone();
        let form_image = form_image;
        let uploading = uploading;
        let toast = toast;
        move |_| {
            if let Some((bytes, filename, mime)) = pending_upload.get() {
                let id = get_id();
                let c = client.clone();
                let fi = form_image;
                let u = uploading;
                let t = toast;
                pending_upload.set(None);
                if let Some(ref id_val) = id {
                    let id_clone = id_val.clone();
                    u.set(true);
                    spawn_local(async move {
                        match crate::modules::oggetti_astronomici::api::upload_oggetto_image(
                            &c, &id_clone, bytes, &filename, &mime,
                        )
                        .await
                        {
                            Ok(o) => {
                                fi.set(o.image_filename.clone());
                                toast_success(&t, "Immagine aggiornata");
                            }
                            Err(e) => toast_error(&t, &e.to_string()),
                        }
                        u.set(false);
                    });
                }
            }
        }
    });

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
                            Ok(()) => {
                                toast_success(&toast, "Oggetto astronomico eliminato");
                                let _ = navigate("/oggetti_astronomici", Default::default());
                            }
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
                tipo: match form_tipo.get().as_str() {
                    "OpC" => TipoOggetto::AmmassoAperto,
                    "GCl" => TipoOggetto::AmmassoGlobulare,
                    "Neb" => TipoOggetto::Nebulosa,
                    "PN" => TipoOggetto::NebulosaPlanetaria,
                    "SNR" => TipoOggetto::RestoSupernova,
                    "Star" => TipoOggetto::Stella,
                    _ => TipoOggetto::Galassia,
                },
                nome_comune: form_nome.get(),
                abbr_costellazione: Costellazione::parse(&form_costellazione.get()),
                coord_ar: form_ar.get(),
                coord_dec: form_dec.get(),
                mag_apparente: form_mag.get().parse().ok(),
                dim_apparenti: None,
                note: form_note.get(),
                multi: false,
                imported: false,
                cataloghi: Vec::new(),
            };
            let toast = toast.clone();
            spawn_local({
                let client = client.clone();
                async move {
                    if let Some(ref id_val) = id {
                        match crate::modules::oggetti_astronomici::api::update_oggetto(&client, id_val, &body).await {
                            Ok(o) => {
                                oggetto.set(Some(o));
                                toast_success(&toast, "Oggetto astronomico aggiornato");
                            }
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
                let file_clone = file.clone();
                let reader = FileReader::new().unwrap();
                let pu = pending_upload;
                let onload: Closure<dyn FnMut(web_sys::ProgressEvent)> =
                    Closure::new(Box::new(move |pe: web_sys::ProgressEvent| {
                        let reader: FileReader = pe.target().unwrap().dyn_into().unwrap();
                        if let Ok(result) = reader.result() {
                            let array = Uint8Array::new(&result);
                            let mut bytes = vec![0u8; array.length() as usize];
                            array.copy_to(&mut bytes);
                            let fname = file_clone.name();
                            let mt = mime.clone();
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

    let image_url = Signal::derive(move || {
        form_image.get().map(|f| format!("/api/filemanager/{}", f))
    });

    view! {
        <Title text="Dettaglio oggetto astronomico - App Gateway"/>

        <div class="mx-auto space-y-8">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <a
                        href="/oggetti_astronomici"
                        class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground h-8 w-8 border border-input bg-background"
                    >
                        {Icon::ArrowLeft.render()}
                    </a>
                    <div>
                        <h2 class="text-xl font-semibold text-foreground mb-1">
                            {move || form_nome.get()}
                        </h2>
                        <p class="text-sm text-muted-foreground">
                            "Dettaglio e modifica oggetto astronomico"
                        </p>
                    </div>
                </div>
                <Button
                    variant=ButtonVariant::Destructive
                    size=ButtonSize::Icon
                    on_click=Arc::new(move || delete_open.set(true))
                >
                    {Icon::Trash.render()}
                </Button>
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6">
                <h3 class="text-lg font-medium text-foreground mb-4">"Dati oggetto"</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Nome comune"</label>
                            <input
                                type="text"
                                prop:value=form_nome
                                on:input=move |e| form_nome.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Tipo"</label>
                            <select
                                prop:value=form_tipo
                                on:change=move |e| form_tipo.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                            >
                                <option value="GAL">Galassia</option>
                                <option value="OpC">Ammasso aperto</option>
                                <option value="GCl">Ammasso globulare</option>
                                <option value="Neb">Nebulosa</option>
                                <option value="PN">Nebulosa planetaria</option>
                                <option value="SNR">Resto di supernova</option>
                                <option value="Star">Stella</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Costellazione"</label>
                            <input
                                type="text"
                                prop:value=form_costellazione
                                on:input=move |e| form_costellazione.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"AR (J2000)"</label>
                            <input
                                type="text"
                                prop:value=form_ar
                                on:input=move |e| form_ar.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"DEC (J2000)"</label>
                            <input
                                type="text"
                                prop:value=form_dec
                                on:input=move |e| form_dec.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Mag. apparente"</label>
                            <input
                                type="text"
                                prop:value=form_mag
                                on:input=move |e| form_mag.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Note"</label>
                            <textarea
                                prop:value=form_note
                                on:input=move |e| form_note.set(event_target_value(&e))
                                class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm min-h-[80px]"
                            />
                        </div>
                    </div>

                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-foreground mb-1">"Immagine"</label>
                            <div class="flex flex-col items-center gap-3 p-4 rounded-md border border-dashed border-border">
                                {move || {
                                    let url = image_url.get();
                                    if let Some(url) = url {
                                        view! {
                                            <img
                                                src=url
                                                class="max-w-full max-h-[200px] rounded-md object-contain"
                                                alt="Oggetto astronomico"
                                            />
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="flex flex-col items-center gap-2 text-muted-foreground py-8">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
                                                <span class="text-sm">"Nessuna immagine"</span>
                                            </div>
                                        }.into_any()
                                    }
                                }}
                                <label class="w-full">
                                    <input
                                        type="file"
                                        accept="image/*"
                                        class="hidden"
                                        on:change=handle_file_select
                                    />
                                    <span class="block w-full text-center px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors cursor-pointer">
                                        {move || if uploading.get() { "Caricamento..." } else { "Carica immagine" }}
                                    </span>
                                </label>
                            </div>
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
                item_type="oggetto astronomico"
                on_confirm=on_delete
            />
        </div>
    }
}
