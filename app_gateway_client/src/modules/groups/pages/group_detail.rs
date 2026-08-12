use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::modules::base::toast_utils::{toast_error, toast_success, use_toast_ctx};
use crate::modules::base::traits::CrudApi;
use crate::modules::groups::api::GroupsApi;
use crate::stores::auth_store::use_auth;
use app_modules::base::groups::models::{GroupPermission, GroupUpdate};
use valerios_ui_toolkit::button::{Button, ButtonVariant};
use valerios_ui_toolkit::confirm_delete::ConfirmDeleteDialog;
use valerios_ui_toolkit::icon::Icon;

#[component]
pub fn GroupDetail() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let toast = use_toast_ctx();
    let groups_api: Arc<GroupsApi> = Arc::new(GroupsApi::new(auth.get_api_client()));

    let params = use_params_map();
    let get_id = move || params.get().get("id").map(|s| s.to_string());

    let group = RwSignal::new(None::<app_modules::base::groups::models::Group>);

    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let permissions = RwSignal::new(Vec::<GroupPermission>::new());

    let delete_open = RwSignal::new(false);

    let is_default =
        Signal::derive(move || group.get().as_ref().map(|g| g.default).unwrap_or(false));

    {
        let groups_api = groups_api.clone();
        let id = get_id();
        spawn_local(async move {
            if let Some(ref id_val) = id {
                match groups_api.get(&id_val).await {
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

    let on_delete = {
        let toast = toast.clone();
        let navigate = navigate.clone();
        let get_id = get_id.clone();
        let groups_api = Arc::clone(&groups_api);

        Callback::new(move |_| {
            let id = get_id();
            let navigate = navigate.clone();
            let toast = toast.clone();
            let groups_api = Arc::clone(&groups_api);
            spawn_local({
                async move {
                    if let Some(ref id_val) = id {
                        match groups_api.delete(&id_val).await {
                            Ok(()) => {
                                toast_success(&toast, "Gruppo eliminato");
                                let _ = navigate("/settings/groups", Default::default());
                            }
                            Err(e) => toast_error(&toast, &e.to_string()),
                        }
                    }
                }
            });
        })
    };

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
                <div class:hidden=is_default>
                    <Button
                        variant=ButtonVariant::Destructive
                        size=valerios_ui_toolkit::button::ButtonSize::Icon
                        on_click=Arc::new(move || delete_open.set(true))
                    >
                        {Icon::Trash.render()}
                    </Button>
                </div>
            </div>

            <div class="bg-background rounded-lg border border-border shadow-sm p-6 space-y-4">
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Nome gruppo"</label>
                    <input type="text" prop:value=name on:input=move |e| name.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                </div>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Descrizione"</label>
                    <input type="text" prop:value=description on:input=move |e| description.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                </div>

                <div>
                    <div class="flex items-center gap-2 mb-2">
                        <label class="block text-sm font-medium text-foreground">"Permessi"</label>
                        {move || if is_default.get() {
                            view! {
                                <span class="inline-flex items-center gap-1 rounded-md border border-amber-200 bg-amber-50 px-2 py-0.5 text-xs font-medium text-amber-700 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-400">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
                                    "Permessi in sola lettura"
                                </span>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                    <div class="border border-border rounded-md overflow-hidden">
                        <table class="w-full text-sm text-foreground">
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
                                    let locked = is_default.get();
                                    perms.into_iter().map(|p| {
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
                                                                disabled=locked
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
                                                                class="rounded border-primary"/>
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
                        move |_| {
                            let id = get_id();
                            let body = GroupUpdate {
                                name: name.get(), description: Some(description.get()),
                                permissions: permissions.get(),
                            };
                            let toast = toast.clone();
                            let groups_api : Arc<GroupsApi> = Arc::clone(&groups_api);
                            spawn_local({
                                async move {
                                    if let Some(ref id_val) = id {
                                        match groups_api.update(&id_val, &body).await {
                                            Ok(g) => { group.set(Some(g)); toast_success(&toast, "Gruppo aggiornato"); }
                                            Err(e) => toast_error(&toast, &e.to_string()),
                                        }
                                    }
                                }
                            });
                        }
                    }
                        class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors">
                        "Salva modifiche"
                    </button>
                </div>
            </div>

            <ConfirmDeleteDialog open=delete_open item_type="gruppo" on_confirm=on_delete />
        </div>
    }
}
