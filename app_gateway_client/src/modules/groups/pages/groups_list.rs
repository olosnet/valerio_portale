#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use crate::modules::base::toast_utils::{use_toast_ctx, toast_error, toast_success};
use crate::modules::groups::models::{Group, GroupCreate};
use crate::stores::auth_store::use_auth;

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
                    <button type="button" on:click=move |_| on_create.run(())
                        class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity">
                        "Crea"
                    </button>
                    <DialogClose>
                        <button type="button"
                            class="px-4 py-2 rounded-md border border-input bg-background text-foreground text-sm hover:bg-secondary transition-colors">
                            "Annulla"
                        </button>
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
                view! { <span class="text-green-600 text-xs font-medium">"S&igrave;"</span> }.into_any()
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
                    class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity">
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
