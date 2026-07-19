#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use crate::modules::base::toast_utils::{use_toast_ctx, toast_error, toast_success};
use crate::modules::groups::models::{Group, GroupCreate};
use crate::stores::auth_store::use_auth;

use valerios_ui_toolkit::button::{Button, ButtonVariant};
use valerios_ui_toolkit::data_table::{ColumnDef, DataTable, DataTableSource};
use valerios_ui_toolkit::dialog::{Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogClose};

#[component]
fn CreateGroupDialog(
    open: RwSignal<bool>,
    new_name: RwSignal<String>,
    new_description: RwSignal<String>,
    on_create: Callback<()>,
) -> impl IntoView {
    view! {
        <Dialog open=open>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>"Crea nuovo gruppo"</DialogTitle>
                </DialogHeader>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Nome"</label>
                    <input type="text" prop:value=new_name
                        on:input=move |e| new_name.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                </div>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Descrizione"</label>
                    <input type="text" prop:value=new_description
                        on:input=move |e| new_description.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                </div>
                <DialogFooter>
                    <Button on_click=Arc::new(move || on_create.run(()))>"Crea"</Button>
                    <DialogClose>
                        <Button variant=ButtonVariant::Outline>"Annulla"</Button>
                    </DialogClose>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    }
}

#[component]
pub fn GroupsList() -> impl IntoView {
    let auth = use_auth();
    let toast = use_toast_ctx();
    let navigate = use_navigate();
    let client = auth.api_client.clone();

    let groups: RwSignal<Vec<Group>> = RwSignal::new(Vec::new());
    let loading = RwSignal::new(true);
    let create_open = RwSignal::new(false);

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

    let on_create = Callback::new({
        let client = client.clone();
        let toast = toast;
        let groups = groups;
        move |_: ()| {
            let body = GroupCreate {
                name: Some(new_name.get()),
                description: Some(new_description.get()),
                permissions: Vec::new(),
            };
            let t = toast;
            let g = groups;
            spawn_local({
                let client = client.clone();
                async move {
                    match crate::modules::groups::api::create_group(&client, &body).await {
                        Ok(grp) => {
                            g.update(|list| list.push(grp));
                            new_name.set(String::new());
                            new_description.set(String::new());
                            toast_success(&t, "Gruppo creato");
                        }
                        Err(e) => toast_error(&t, &e.to_string()),
                    }
                }
            });
        }
    });

    let columns = vec![
        ColumnDef {
            title: "Nome",
            sortable: true,
            searchable: true,
            cell: Arc::new(|g: &Group| g.name.clone().unwrap_or_default().into_any()),
            sort_key: Some(Arc::new(|g| g.name.clone().unwrap_or_default())),
            search_key: Some(Arc::new(|g| g.name.clone().unwrap_or_default())),
        },
        ColumnDef {
            title: "Descrizione",
            sortable: true,
            searchable: true,
            cell: Arc::new(|g: &Group| g.description.clone().unwrap_or_default().into_any()),
            sort_key: Some(Arc::new(|g| g.description.clone().unwrap_or_default())),
            search_key: Some(Arc::new(|g| g.description.clone().unwrap_or_default())),
        },
        ColumnDef {
            title: "Permessi",
            sortable: false,
            searchable: false,
            cell: Arc::new(|g: &Group| g.permissions.len().to_string().into_any()),
            sort_key: None,
            search_key: None,
        },
        ColumnDef {
            title: "Default",
            sortable: true,
            searchable: false,
            cell: Arc::new(|g: &Group| if g.default {
                view! {
                    <span class="inline-flex items-center gap-1 rounded-md border border-amber-200 bg-amber-50 px-2 py-0.5 text-xs font-medium text-amber-700 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-400">
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
                        "Predefinito"
                    </span>
                }.into_any()
            } else {
                view! { <span class="text-muted-foreground text-xs">"No"</span> }.into_any()
            }),
            sort_key: Some(Arc::new(|g| g.default.to_string())),
            search_key: None,
        },
    ];

    let actions = {
        let nav = navigate;
        Arc::new(move |g: &Group| {
            let gid = g.id.clone().unwrap_or_default();
            let nav = nav.clone();
            view! {
                <button type="button" on:click=move |_| {
                    let _ = nav(&format!("/settings/groups/{gid}"), Default::default());
                }
                    class="text-sm text-primary underline hover:no-underline">
                    "Dettaglio"
                </button>
            }.into_any()
        })
    };

    view! {
        <Title text="Gruppi - App Gateway"/>

        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-semibold text-foreground mb-1">"Gruppi"</h2>
                    <p class="text-sm text-muted-foreground">"Gestisci i gruppi e i relativi permessi"</p>
                </div>
                <button type="button" on:click=move |_| create_open.set(true)
                    class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors">
                    "Nuovo gruppo"
                </button>
            </div>

            {move || {
                if loading.get() {
                    view! { <p class="text-sm text-muted-foreground">"Caricamento..."</p> }.into_any()
                } else {
                    view! {
                        <DataTable
                            source=DataTableSource::Client(groups.get())
                            columns=columns.clone()
                            initial_page_size=10
                            show_search=true
                            actions=actions.clone()
                        />
                    }.into_any()
                }
            }}
        </div>

        <CreateGroupDialog
            open=create_open
            new_name=new_name
            new_description=new_description
            on_create=on_create
        />
    }
}
