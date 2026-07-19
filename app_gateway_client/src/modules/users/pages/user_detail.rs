use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::modules::base::toast_utils::{use_toast_ctx, toast_error, toast_success};
use crate::modules::groups::models::Group;
use crate::modules::users::models::{SetPasswordBody, UserUpdate};
use crate::stores::auth_store::use_auth;
use valerios_ui_toolkit::icon::Icon;

#[component]
pub fn UserDetail() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let toast = use_toast_ctx();
    let client = auth.api_client.clone();
    let params = use_params_map();
    let get_id = move || params.get().get("id").map(|s| s.to_string());

    let user = RwSignal::new(None::<crate::modules::users::models::User>);
    let groups = RwSignal::new(Vec::<Group>::new());
    let password_saved = RwSignal::new(false);

    let name = RwSignal::new(String::new());
    let surname = RwSignal::new(String::new());
    let enabled = RwSignal::new(true);
    let selected_groups = RwSignal::new(Vec::<String>::new());

    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());

    {
        let client = client.clone();
        let id = get_id();
        spawn_local(async move {
            if let Some(ref id_val) = id {
                match crate::modules::users::api::get_user(&client, id_val).await {
                    Ok(u) => {
                        name.set(u.name.clone().unwrap_or_default());
                        surname.set(u.surname.clone().unwrap_or_default());
                        enabled.set(u.enabled);
                        selected_groups.set(u.groups_ids.clone());
                        user.set(Some(u));
                    }
                    Err(e) => toast_error(&toast, &e.to_string()),
                }
                match crate::modules::groups::api::list_groups(&client).await {
                    Ok(g) => groups.set(g),
                    Err(e) => toast_error(&toast, &e.to_string()),
                }
            }
        });
    }

    view! {
        <Title text="Dettaglio utente - App Gateway"/>

        <div class="max-w-2xl mx-auto space-y-8">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <a href="/settings/users"
                        class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground h-8 w-8 border border-input bg-background">
                        {Icon::ArrowLeft.render()}
                    </a>
                    <div>
                        <h2 class="text-xl font-semibold text-foreground mb-1">
                            {move || user.get().as_ref().and_then(|u| u.name.clone()).unwrap_or_default()}
                            " "
                            {move || user.get().as_ref().and_then(|u| u.surname.clone()).unwrap_or_default()}
                        </h2>
                        <p class="text-sm text-muted-foreground">
                            {move || user.get().as_ref().and_then(|u| u.email.clone()).unwrap_or_default()}
                        </p>
                    </div>
                </div>
                <button on:click={
                    let client = client.clone();
                    let navigate = navigate.clone();
                    move |_| {
                        let id = get_id();
                        let navigate = navigate.clone();
                        spawn_local({
                            let client = client.clone();
                            async move {
                                if let Some(ref id_val) = id {
                                    let _ = crate::modules::users::api::delete_user(&client, id_val).await;
                                    let _ = navigate("/settings/users", Default::default());
                                }
                            }
                        });
                    }
                }
                    class="px-3 py-2 rounded-md bg-destructive text-destructive-foreground text-sm hover:opacity-90 transition-opacity">
                    "Elimina"
                </button>
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6 space-y-4">
                <h3 class="text-lg font-medium text-foreground">"Modifica dati"</h3>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-1">"Nome"</label>
                        <input type="text" prop:value=name on:input=move |e| name.set(event_target_value(&e))
                            class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-1">"Cognome"</label>
                        <input type="text" prop:value=surname on:input=move |e| surname.set(event_target_value(&e))
                            class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                    </div>
                </div>

                <div class="flex items-center gap-2">
                    <input type="checkbox" id="enabled" checked=enabled
                        on:change=move |e| enabled.set(event_target_checked(&e))
                        class="rounded border-border"/>
                    <label for="enabled" class="text-sm text-foreground">"Utente abilitato"</label>
                </div>

                <div>
                    <label class="block text-sm font-medium text-foreground mb-2">"Gruppi"</label>
                    <div class="space-y-2">
                        {move || {
                            let all_groups = groups.get();
                            let selected = selected_groups.get();
                            all_groups.into_iter().map(|g| {
                                let g_id = g.id.clone().unwrap_or_default();
                                let g_name = g.name.clone().unwrap_or_default();
                                let is_checked = selected.contains(&g_id);
                                view! {
                                    <label class="flex items-center gap-2 text-sm">
                                        <input type="checkbox" checked=is_checked
                                            on:change=move |e| {
                                                let mut sel = selected_groups.get();
                                                if event_target_checked(&e) {
                                                    if !sel.contains(&g_id) { sel.push(g_id.clone()); }
                                                } else {
                                                    sel.retain(|x| x != &g_id);
                                                }
                                                selected_groups.set(sel);
                                            }
                                            class="rounded border-border"/>
                                        {g_name}
                                    </label>
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </div>
                </div>

                <div class="flex items-center justify-between">
                    <button on:click={
                        let client = client.clone();
                        move |_| {
                            let id = get_id();
                            let body = UserUpdate {
                                name: name.get(), surname: surname.get(),
                                enabled: enabled.get(), groups_ids: selected_groups.get(),
                            };
                            let toast = toast.clone();
                            spawn_local({
                                let client = client.clone();
                                async move {
                                    if let Some(ref id_val) = id {
                                        match crate::modules::users::api::update_user(&client, id_val, &body).await {
                                            Ok(u) => { user.set(Some(u)); toast_success(&toast, "Utente aggiornato"); }
                                            Err(e) => toast_error(&toast, &e.to_string()),
                                        }
                                    }
                                }
                            });
                        }
                    }
                        class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity">
                        "Salva modifiche"
                    </button>
                </div>
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6 space-y-4">
                <h3 class="text-lg font-medium text-foreground">"Imposta password"</h3>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Nuova password"</label>
                    <input type="password" prop:value=password
                        on:input=move |e| password.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                </div>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Conferma password"</label>
                    <input type="password" prop:value=confirm_password
                        on:input=move |e| confirm_password.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                </div>
                <div class="flex items-center justify-between">
                    <button on:click={
                        let client = client.clone();
                        move |_| {
                            let id = get_id();
                            let body = SetPasswordBody {
                                password: password.get(), confirm_password: confirm_password.get(),
                            };
                            let toast = toast.clone();
                            spawn_local({
                                let client = client.clone();
                                async move {
                                    if let Some(ref id_val) = id {
                                        match crate::modules::users::api::set_password(&client, id_val, &body).await {
                                            Ok(u) => {
                                                user.set(Some(u));
                                                password.set(String::new());
                                                confirm_password.set(String::new());
                                                toast_success(&toast, "Password impostata");
                                            }
                                            Err(e) => toast_error(&toast, &e.to_string()),
                                        }
                                    }
                                }
                            });
                        }
                    }
                        class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity">
                        "Imposta password"
                    </button>
                </div>
            </div>
        </div>
    }
}
