use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::modules::base::toast_utils::{use_toast_ctx, toast_error, toast_success};
use crate::modules::groups::models::{GroupPermission, GroupUpdate};
use crate::stores::auth_store::use_auth;
use valerios_ui_toolkit::icon::Icon;

#[component]
pub fn GroupDetail() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let toast = use_toast_ctx();
    let client = auth.api_client.clone();
    let params = use_params_map();
    let get_id = move || params.get().get("id").map(|s| s.to_string());

    let group = RwSignal::new(None::<crate::modules::groups::models::Group>);
    let permissions = RwSignal::new(Vec::<GroupPermission>::new());

    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let permissions = RwSignal::new(Vec::<GroupPermission>::new());

    {
        let client = client.clone();
        let id = get_id();
        spawn_local(async move {
            if let Some(ref id_val) = id {
                match crate::modules::groups::api::get_group(&client, id_val).await {
                    Ok(g) => {
                        name.set(g.name.clone().unwrap_or_default());
                        description.set(g.description.clone().unwrap_or_default());
                        permissions.set(g.permissions.clone());
                        group.set(Some(g));
                    }
                    Err(e) => toast_error(&toast, &e.to_string()),
                }
            }
        });
    }

    view! {
        <Title text="Dettaglio gruppo - App Gateway"/>

        <div class="max-w-2xl mx-auto space-y-8">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <a href="/settings/groups"
                        class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground h-8 w-8 border border-input bg-background">
                        {Icon::ArrowLeft.render()}
                    </a>
                    <h2 class="text-xl font-semibold text-foreground">
                        {move || group.get().as_ref().and_then(|g| g.name.clone()).unwrap_or_default()}
                    </h2>
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
                                    let _ = crate::modules::groups::api::delete_group(&client, id_val).await;
                                    let _ = navigate("/settings/groups", Default::default());
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
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Nome gruppo"</label>
                    <input type="text" prop:value=name on:input=move |e| name.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                </div>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Descrizione"</label>
                    <input type="text" prop:value=description on:input=move |e| description.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm"/>
                </div>

                <div>
                    <label class="block text-sm font-medium text-foreground mb-2">"Permessi"</label>
                    <div class="border border-border rounded-md overflow-hidden">
                        <table class="w-full text-sm">
                            <thead>
                                <tr class="border-b border-border bg-muted/50">
                                    <th class="text-left px-4 py-2 font-medium text-foreground">"Modulo"</th>
                                    <th class="text-center px-2 py-2 font-medium text-foreground">"Lettura"</th>
                                    <th class="text-center px-2 py-2 font-medium text-foreground">"Creazione"</th>
                                    <th class="text-center px-2 py-2 font-medium text-foreground">"Modifica"</th>
                                    <th class="text-center px-2 py-2 font-medium text-foreground">"Eliminazione"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    let perms = permissions.get();
                                    perms.into_iter().map(|mut p| {
                                        let p_name = p.name.clone();
                                        view! {
                                            <tr class="border-b border-border hover:bg-muted/30 transition-colors">
                                                <td class="px-4 py-2 text-foreground font-medium">{p_name.clone()}</td>
                                                {["read", "create", "modify", "delete"].iter().map(|&field| {
                                                    let p_name = p_name.clone();
                                                    let checked = match field {
                                                        "read" => p.read, "create" => p.create,
                                                        "modify" => p.modify, "delete" => p.delete,
                                                        _ => false,
                                                    };
                                                    view! {
                                                        <td class="px-2 py-2 text-center">
                                                            <input type="checkbox" checked=checked
                                                                on:change=move |e| {
                                                                    let val = event_target_checked(&e);
                                                                    permissions.update(|list| {
                                                                        if let Some(perm) = list.iter_mut().find(|x| x.name == p_name) {
                                                                            match field {
                                                                                "read" => perm.read = val,
                                                                                "create" => perm.create = val,
                                                                                "modify" => perm.modify = val,
                                                                                "delete" => perm.delete = val,
                                                                                _ => {}
                                                                            }
                                                                        }
                                                                    });
                                                                }
                                                                class="rounded border-border"/>
                                                        </td>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()
                                }}
                            </tbody>
                        </table>
                    </div>
                </div>

                <div class="flex items-center justify-between">
                    <button on:click={
                        let client = client.clone();
                        move |_| {
                            let id = get_id();
                            let body = GroupUpdate {
                                name: name.get(), description: Some(description.get()),
                                permissions: permissions.get(),
                            };
                            let toast = toast.clone();
                            spawn_local({
                                let client = client.clone();
                                async move {
                                    if let Some(ref id_val) = id {
                                        match crate::modules::groups::api::update_group(&client, id_val, &body).await {
                                            Ok(g) => { group.set(Some(g)); toast_success(&toast, "Gruppo aggiornato"); }
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
        </div>
    }
}
