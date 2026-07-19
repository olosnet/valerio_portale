use leptos::prelude::*;
use leptos_meta::Title;

use crate::modules::base::toast_utils::{use_toast_ctx, toast_error, toast_success};
use crate::modules::identity::models::{UserIdentityUpdate, UserIdentityUpdatePassword};
use crate::stores::auth_store::use_auth;

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
                            class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-1">"Cognome"</label>
                        <input
                            type="text"
                            prop:value=surname
                            on:input=move |e| surname.set(event_target_value(&e))
                            class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"
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
                        class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity"
                    >
                        "Salva"
                    </button>
                </div>
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6 space-y-4">
                <h3 class="text-lg font-medium text-foreground">"Cambia password"</h3>

                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Password attuale"</label>
                    <input type="password" prop:value=old_password
                        on:input=move |e| old_password.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                </div>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Nuova password"</label>
                    <input type="password" prop:value=new_password
                        on:input=move |e| new_password.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                </div>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Conferma password"</label>
                    <input type="password" prop:value=confirm_password
                        on:input=move |e| confirm_password.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                </div>

                <div class="flex items-center justify-between">
                    <button on:click=save_password
                        class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity">
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
    }
}
