use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use crate::modules::users::models::{User, UserCreate};
use crate::stores::auth_store::use_auth;

#[component]
pub fn UsersList() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let client = auth.api_client.clone();

    let users = RwSignal::new(Vec::<User>::new());
    let loading = RwSignal::new(true);
    let show_create = RwSignal::new(false);

    let new_name = RwSignal::new(String::new());
    let new_surname = RwSignal::new(String::new());
    let new_email = RwSignal::new(String::new());
    let new_enabled = RwSignal::new(true);

    {
        let client = client.clone();
        spawn_local(async move {
            match crate::modules::users::api::list_users(&client).await {
                Ok(list) => users.set(list),
                Err(_) => {}
            }
            loading.set(false);
        });
    }

    view! {
        <Title text="Utenti - App Gateway"/>

        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-semibold text-foreground mb-1">"Utenti"</h2>
                    <p class="text-sm text-muted-foreground">"Gestisci gli utenti della piattaforma"</p>
                </div>
                <button
                    on:click=move |_| show_create.set(true)
                    class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity"
                >
                    "Nuovo utente"
                </button>
            </div>

            {move || {
                if show_create.get() {
                    view! {
                        <div class="bg-background rounded-lg border border-border shadow-sm p-6 space-y-4 mb-6">
                            <h3 class="text-lg font-medium text-foreground">"Crea nuovo utente"</h3>
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class="block text-sm font-medium text-foreground mb-1">"Nome"</label>
                                    <input type="text" prop:value=new_name
                                        on:input=move |e| new_name.set(event_target_value(&e))
                                        class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-foreground mb-1">"Cognome"</label>
                                    <input type="text" prop:value=new_surname
                                        on:input=move |e| new_surname.set(event_target_value(&e))
                                        class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                                </div>
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-foreground mb-1">"Email"</label>
                                <input type="email" prop:value=new_email
                                    on:input=move |e| new_email.set(event_target_value(&e))
                                    class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                            </div>
                            <div class="flex items-center gap-2">
                                <input type="checkbox" checked=new_enabled
                                    on:change=move |e| new_enabled.set(event_target_checked(&e))
                                    class="rounded border-border"/>
                                <label class="text-sm text-foreground">"Abilitato"</label>
                            </div>
                            <div class="flex gap-2">
                                <button on:click={
                                    let client = client.clone();
                                    move |_| {
                                        let body = UserCreate {
                                            name: new_name.get(),
                                            surname: new_surname.get(),
                                            email: new_email.get(),
                                            enabled: new_enabled.get(),
                                            groups_ids: Vec::new(),
                                        };
                                        spawn_local({
                                            let client = client.clone();
                                            async move {
                                                if let Ok(u) = crate::modules::users::api::create_user(&client, &body).await {
                                                    users.update(|list| list.push(u));
                                                    show_create.set(false);
                                                    new_name.set(String::new());
                                                    new_surname.set(String::new());
                                                    new_email.set(String::new());
                                                    new_enabled.set(true);
                                                }
                                            }
                                        });
                                    }
                                }
                                    class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity">
                                    "Crea"
                                </button>
                                <button on:click=move |_| show_create.set(false)
                                    class="px-4 py-2 rounded-md border border-border bg-background text-foreground text-sm hover:bg-secondary transition-colors">
                                    "Annulla"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}

            {move || {
                if loading.get() {
                    view! { <p class="text-sm text-muted-foreground">"Caricamento..."</p> }.into_any()
                } else if users.get().is_empty() {
                    view! { <p class="text-sm text-muted-foreground">"Nessun utente trovato"</p> }.into_any()
                } else {
                    let items = users.get();
                    view! {
                        <div class="bg-background rounded-lg border border-border shadow-sm overflow-hidden">
                            <table class="w-full text-sm">
                                <thead>
                                    <tr class="border-b border-border bg-muted/50">
                                        <th class="text-left px-4 py-3 font-medium text-foreground">"Nome"</th>
                                        <th class="text-left px-4 py-3 font-medium text-foreground">"Cognome"</th>
                                        <th class="text-left px-4 py-3 font-medium text-foreground">"Email"</th>
                                        <th class="text-center px-4 py-3 font-medium text-foreground">"Abilitato"</th>
                                        <th class="text-right px-4 py-3 font-medium text-foreground">"Azioni"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {items.into_iter().map(|u| {
                                        let uid = u.id.clone().unwrap_or_default();
                                        let navigate = navigate.clone();
                                        view! {
                                            <tr class="border-b border-border hover:bg-muted/30 transition-colors">
                                                <td class="px-4 py-3 text-foreground">{u.name.clone().unwrap_or_default()}</td>
                                                <td class="px-4 py-3 text-foreground">{u.surname.clone().unwrap_or_default()}</td>
                                                <td class="px-4 py-3 text-muted-foreground">{u.email.clone().unwrap_or_default()}</td>
                                                <td class="px-4 py-3 text-center">
                                                    {if u.enabled {
                                                        view! { <span class="text-green-600 text-xs font-medium">"Sì"</span> }.into_any()
                                                    } else {
                                                        view! { <span class="text-destructive text-xs font-medium">"No"</span> }.into_any()
                                                    }}
                                                </td>
                                                <td class="px-4 py-3 text-right">
                                                    <button
                                                        on:click=move |_| {
                                                            let _ = navigate(&format!("/settings/users/{uid}"), Default::default());
                                                        }
                                                        class="text-sm text-primary underline hover:no-underline"
                                                    >
                                                        "Dettaglio"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
