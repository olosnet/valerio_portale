use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use crate::modules::base::toast_utils::{use_toast_ctx, toast_error, toast_success};
use crate::modules::groups::models::{Group, GroupCreate};
use crate::stores::auth_store::use_auth;

#[component]
pub fn GroupsList() -> impl IntoView {
    let auth = use_auth();
    let toast = use_toast_ctx();
    let navigate = use_navigate();
    let client = auth.api_client.clone();

    let groups = RwSignal::new(Vec::<Group>::new());
    let loading = RwSignal::new(true);
    let show_create = RwSignal::new(false);

    let new_name = RwSignal::new(String::new());
    let new_description = RwSignal::new(String::new());

    {
        let client = client.clone();
        spawn_local(async move {
            match crate::modules::groups::api::list_groups(&client).await {
                Ok(list) => groups.set(list),
                Err(e) => toast_error(&toast, &e.to_string()),
            }
            loading.set(false);
        });
    }

    view! {
        <Title text="Gruppi - App Gateway"/>

        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-semibold text-foreground mb-1">"Gruppi"</h2>
                    <p class="text-sm text-muted-foreground">"Gestisci i gruppi e i relativi permessi"</p>
                </div>
                <button
                    on:click=move |_| show_create.set(true)
                    class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity"
                >
                    "Nuovo gruppo"
                </button>
            </div>

            {move || {
                if show_create.get() {
                    view! {
                        <div class="bg-background rounded-lg border border-border shadow-sm p-6 space-y-4 mb-6">
                            <h3 class="text-lg font-medium text-foreground">"Crea nuovo gruppo"</h3>
                            <div>
                                <label class="block text-sm font-medium text-foreground mb-1">"Nome"</label>
                                <input type="text" prop:value=new_name
                                    on:input=move |e| new_name.set(event_target_value(&e))
                                    class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-foreground mb-1">"Descrizione"</label>
                                <input type="text" prop:value=new_description
                                    on:input=move |e| new_description.set(event_target_value(&e))
                                    class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                            </div>
                            <div class="flex gap-2">
                                <button on:click={
                                    let client = client.clone();
                                    move |_| {
                                        let body = GroupCreate {
                                            name: Some(new_name.get()),
                                            description: Some(new_description.get()),
                                            permissions: Vec::new(),
                                            };
                                            let toast = toast.clone();
                                            spawn_local({
                                                let client = client.clone();
                                                async move {
                                                    match crate::modules::groups::api::create_group(&client, &body).await {
                                                    Ok(g) => {
                                                        groups.update(|list| list.push(g));
                                                        show_create.set(false);
                                                        new_name.set(String::new());
                                                        new_description.set(String::new());
                                                        toast_success(&toast, "Gruppo creato");
                                                    }
                                                    Err(e) => toast_error(&toast, &e.to_string()),
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
                } else if groups.get().is_empty() {
                    view! { <p class="text-sm text-muted-foreground">"Nessun gruppo trovato"</p> }.into_any()
                } else {
                    let items = groups.get();
                    view! {
                        <div class="bg-background rounded-lg border border-border shadow-sm overflow-hidden">
                            <table class="w-full text-sm">
                                <thead>
                                    <tr class="border-b border-border bg-muted/50">
                                        <th class="text-left px-4 py-3 font-medium text-foreground">"Nome"</th>
                                        <th class="text-left px-4 py-3 font-medium text-foreground">"Descrizione"</th>
                                        <th class="text-center px-4 py-3 font-medium text-foreground">"Permessi"</th>
                                        <th class="text-center px-4 py-3 font-medium text-foreground">"Default"</th>
                                        <th class="text-right px-4 py-3 font-medium text-foreground">"Azioni"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {items.into_iter().map(|g| {
                                        let gid = g.id.clone().unwrap_or_default();
                                        let navigate = navigate.clone();
                                        view! {
                                            <tr class="border-b border-border hover:bg-muted/30 transition-colors">
                                                <td class="px-4 py-3 text-foreground font-medium">{g.name.clone().unwrap_or_default()}</td>
                                                <td class="px-4 py-3 text-muted-foreground">{g.description.clone().unwrap_or_default()}</td>
                                                <td class="px-4 py-3 text-center text-muted-foreground">{g.permissions.len().to_string()}</td>
                                                <td class="px-4 py-3 text-center">
                                                    {if g.default {
                                                        view! { <span class="text-green-600 text-xs font-medium">"Sì"</span> }.into_any()
                                                    } else {
                                                        view! { <span class="text-muted-foreground text-xs">"No"</span> }.into_any()
                                                    }}
                                                </td>
                                                <td class="px-4 py-3 text-right">
                                                    <button
                                                        on:click=move |_| {
                                                            let _ = navigate(&format!("/settings/groups/{gid}"), Default::default());
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
