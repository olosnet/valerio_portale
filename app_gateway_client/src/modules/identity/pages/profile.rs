use std::sync::Arc;

use leptos::prelude::*;
use leptos_meta::Title;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::FileReader;

use crate::modules::identity::api::profile_image_url;
use crate::modules::base::toast_utils::{use_toast_ctx, toast_error, toast_success};
use crate::modules::identity::models::{UserIdentityUpdate, UserIdentityUpdatePassword};
use crate::stores::auth_store::use_auth;
use valerios_ui_toolkit::button::{Button, ButtonVariant};
use valerios_ui_toolkit::image_cropper::ImageCropper;
use valerios_ui_toolkit::password_input::PasswordInput;

#[component]
pub fn Profile() -> impl IntoView {
    let auth = use_auth();
    let toast = use_toast_ctx();
    let user = auth.user;

    let name = RwSignal::new(String::new());
    let surname = RwSignal::new(String::new());
    let old_password = RwSignal::new(String::new());
    let new_password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());

    let crop_open = RwSignal::new(false);
    let crop_bytes = RwSignal::new(Vec::<u8>::new());
    let uploading = RwSignal::new(false);

    let save_profile = {
        let auth = auth.clone();
        move |_| {
            let auth = auth.clone();
            let body = UserIdentityUpdate {
                name: Some(name.get()),
                surname: Some(surname.get()),
            };
            let toast = toast.clone();
            leptos::task::spawn_local(async move {
                match crate::modules::identity::api::update_profile(&auth.api_client, &body).await {
                    Ok(u) => {
                        auth.user.set(Some(u));
                        toast_success(&toast, "Profilo aggiornato");
                    }
                    Err(e) => toast_error(&toast, &e.to_string()),
                }
            });
        }
    };

    let save_password = {
        let auth = auth.clone();
        move |_| {
            let auth = auth.clone();
            let body = UserIdentityUpdatePassword {
                old_password: old_password.get(),
                new_password: new_password.get(),
                confirm_password: confirm_password.get(),
            };
            let toast = toast.clone();
            leptos::task::spawn_local(async move {
                match crate::modules::identity::api::update_password(&auth.api_client, &body).await {
                    Ok(()) => {
                        old_password.set(String::new());
                        new_password.set(String::new());
                        confirm_password.set(String::new());
                        toast_success(&toast, "Password aggiornata");
                    }
                    Err(e) => toast_error(&toast, &e.to_string()),
                }
            });
        }
    };

    let on_crop = {
        let auth = auth.clone();
        let toast = toast.clone();
        Callback::new(move |cropped: Vec<u8>| {
            let auth = auth.clone();
            let toast = toast.clone();
            uploading.set(true);
            leptos::task::spawn_local(async move {
                match crate::modules::identity::api::upload_profile_image(&auth.api_client, cropped).await {
                    Ok(identity) => {
                        auth.user.set(Some(identity));
                        toast_success(&toast, "Immagine profilo aggiornata");
                        crop_open.set(false);
                    }
                    Err(e) => toast_error(&toast, &e.to_string()),
                }
                uploading.set(false);
            });
        })
    };

    let handle_file_select = {
        move |ev: leptos::ev::Event| {
            let target = event_target::<web_sys::HtmlInputElement>(&ev);
            if let Some(files) = target.files() {
                if let Some(file) = files.get(0) {
                    let reader = FileReader::new().unwrap();
                    let onload: Closure<dyn FnMut(web_sys::ProgressEvent)> = Closure::new(Box::new(move |pe: web_sys::ProgressEvent| {
                        let reader: FileReader = pe.target().unwrap().dyn_into().unwrap();
                        if let Ok(result) = reader.result() {
                            let array = js_sys::Uint8Array::new(&result);
                            let mut bytes = vec![0u8; array.length() as usize];
                            array.copy_to(&mut bytes);
                            crop_bytes.set(bytes);
                            crop_open.set(true);
                        }
                    }));
                    let _ = reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                    let _ = reader.read_as_array_buffer(&file);
                }
                let _ = target.set_value(""); // reset file input
            }
        }
    };

    let open_file_picker = move || {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(input) = document.get_element_by_id("profile-image-input") {
                    if let Some(input_el) = input.dyn_ref::<web_sys::HtmlInputElement>() {
                        let _ = input_el.click();
                    }
                }
            }
        }
    };

    view! {
        <Title text="Il mio profilo - App Gateway"/>

        <div class="max-w-2xl mx-auto space-y-8">
            <div>
                <h2 class="text-xl font-semibold text-foreground mb-1">"Il mio profilo"</h2>
                <p class="text-sm text-muted-foreground">"Gestisci le tue informazioni personali"</p>
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6 space-y-4">
                <h3 class="text-lg font-medium text-foreground">"Dati anagrafici"</h3>

                {move || {
                    let u = user.get();
                    if let Some(ref u) = u {
                        name.set(u.name.clone().unwrap_or_default());
                        surname.set(u.surname.clone().unwrap_or_default());
                    }
                    ().into_any()
                }}

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-1">"Nome"</label>
                        <input
                            type="text"
                            prop:value=name
                            on:input=move |e| name.set(event_target_value(&e))
                            class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-1">"Cognome"</label>
                        <input
                            type="text"
                            prop:value=surname
                            on:input=move |e| surname.set(event_target_value(&e))
                            class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"
                        />
                    </div>
                </div>

                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Email"</label>
                    <input
                        type="email"
                        disabled=true
                        prop:value=move || user.get().as_ref().and_then(|u| u.email.clone()).unwrap_or_default()
                        class="w-full px-3 py-2 rounded-md border border-border bg-muted text-muted-foreground text-sm"
                    />
                </div>

                <div class="flex items-center justify-between">
                    <button
                        on:click=save_profile
                        class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors"
                    >
                        "Salva"
                    </button>
                </div>
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6 space-y-4">
                <h3 class="text-lg font-medium text-foreground">"Immagine profilo"</h3>

                <div class="flex items-center gap-6">
                    <div class="shrink-0">
                        {move || {
                            let u = user.get();
                            let filename = u.as_ref().map(|u| u.profile_image.clone()).unwrap_or_default();
                            let url = profile_image_url("/api", &filename);
                            if url.is_empty() {
                                let initial = u.as_ref().and_then(|u| u.name.as_ref().and_then(|n| n.chars().next().map(|c| c.to_uppercase().to_string()))).unwrap_or_default();
                                view! {
                                    <div class="size-28 rounded-full bg-muted border border-border flex items-center justify-center text-2xl font-bold text-muted-foreground">{initial}</div>
                                }.into_any()
                            } else {
                                view! {
                                    <img src=url class="size-28 rounded-full object-cover border border-border" alt="Immagine profilo" />
                                }.into_any()
                            }
                        }}
                    </div>
                    <div class="space-y-2">
                        <p class="text-sm text-muted-foreground">
                            "Carica una nuova immagine profilo. Verrà ritagliata in formato quadrato."
                        </p>
                        <Button variant=ButtonVariant::Outline on_click=Arc::new(open_file_picker.clone())>
                            {move || if uploading.get() { "Caricamento..." } else { "Cambia immagine" }}
                        </Button>
                    </div>
                </div>

                <input
                    id="profile-image-input"
                    type="file"
                    accept="image/*"
                    class="hidden"
                    on:change=handle_file_select
                />
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6 space-y-4">
                <h3 class="text-lg font-medium text-foreground">"Cambia password"</h3>

                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Password attuale"</label>
                    <PasswordInput value=old_password placeholder="Password attuale" />
                </div>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Nuova password"</label>
                    <PasswordInput value=new_password placeholder="Nuova password" />
                </div>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Conferma password"</label>
                    <PasswordInput value=confirm_password placeholder="Conferma password" />
                </div>

                <div class="flex items-center justify-between">
                    <button on:click=save_password
                        class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors">
                        "Aggiorna password"
                    </button>
                </div>
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6 space-y-2">
                <h3 class="text-lg font-medium text-foreground">"Informazioni account"</h3>
                {move || {
                    match user.get() {
                        Some(ref u) => view! {
                            <div class="text-sm text-muted-foreground space-y-1">
                                <p>"Tipo utente: " {u.user_type.to_string()}</p>
                                <p>"Creato: " {u.created.as_deref().unwrap_or("-")}</p>
                                <p>"Ultimo accesso: " {u.last_access.as_deref().unwrap_or("-")}</p>
                            </div>
                        }.into_any(),
                        None => ().into_any(),
                    }
                }}
            </div>
        </div>

        {move || {
            if !crop_open.get() { return ().into_any(); }
            view! {
                <ImageCropper
                    open=crop_open
                    image_bytes=crop_bytes.get()
                    output_size=256
                    on_crop=on_crop.clone()
                />
            }.into_any()
        }}
    }
}
