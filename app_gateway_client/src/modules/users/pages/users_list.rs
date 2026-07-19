#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use crate::modules::base::toast_utils::{use_toast_ctx, toast_error, toast_success};
use crate::modules::users::models::{User, UserCreate};
use crate::stores::auth_store::use_auth;

use valerios_ui_toolkit::button::{Button, ButtonVariant};
use valerios_ui_toolkit::data_table::{ColumnDef, DataTable, DataTableSource};
use valerios_ui_toolkit::dialog::{Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogClose};

#[component]
fn CreateUserDialog(
    open: RwSignal<bool>,
    new_name: RwSignal<String>,
    new_surname: RwSignal<String>,
    new_email: RwSignal<String>,
    new_enabled: RwSignal<bool>,
    on_create: Callback<()>,
) -> impl IntoView {
    view! {
        <Dialog open=open>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>"Crea nuovo utente"</DialogTitle>
                </DialogHeader>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-1">"Nome"</label>
                        <input type="text" prop:value=new_name
                            on:input=move |e| new_name.set(event_target_value(&e))
                            class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-1">"Cognome"</label>
                        <input type="text" prop:value=new_surname
                            on:input=move |e| new_surname.set(event_target_value(&e))
                            class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                    </div>
                </div>
                <div>
                    <label class="block text-sm font-medium text-foreground mb-1">"Email"</label>
                    <input type="email" prop:value=new_email
                        on:input=move |e| new_email.set(event_target_value(&e))
                        class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                </div>
                <div class="flex items-center gap-2">
                    <input type="checkbox" checked=new_enabled
                        on:change=move |e| new_enabled.set(event_target_checked(&e))
                        class="rounded border-primary"/>
                    <label class="text-sm text-foreground">"Abilitato"</label>
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
pub fn UsersList() -> impl IntoView {
    let auth = use_auth();
    let toast = use_toast_ctx();
    let navigate = use_navigate();
    let client = auth.api_client.clone();

    let users: RwSignal<Vec<User>> = RwSignal::new(Vec::new());
    let loading = RwSignal::new(true);
    let create_open = RwSignal::new(false);

    let new_name = RwSignal::new(String::new());
    let new_surname = RwSignal::new(String::new());
    let new_email = RwSignal::new(String::new());
    let new_enabled = RwSignal::new(true);

    {
        let client = client.clone();
        spawn_local(async move {
            match crate::modules::users::api::list_users(&client).await {
                Ok(list) => users.set(list),
                Err(e) => toast_error(&toast, &e.to_string()),
            }
            loading.set(false);
        });
    }

    let on_create = Callback::new({
        let client = client.clone();
        let toast = toast;
        let users = users;
        move |_: ()| {
            let body = UserCreate {
                name: new_name.get(),
                surname: new_surname.get(),
                email: new_email.get(),
                enabled: new_enabled.get(),
                groups_ids: Vec::new(),
            };
            let t = toast;
            spawn_local({
                let client = client.clone();
                let users = users;
                async move {
                    match crate::modules::users::api::create_user(&client, &body).await {
                        Ok(u) => {
                            users.update(|list| list.push(u));
                            new_name.set(String::new());
                            new_surname.set(String::new());
                            new_email.set(String::new());
                            new_enabled.set(true);
                            toast_success(&t, "Utente creato");
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
            cell: Arc::new(|u: &User| u.name.clone().unwrap_or_default().into_any()),
            sort_key: Some(Arc::new(|u| u.name.clone().unwrap_or_default())),
            search_key: Some(Arc::new(|u| u.name.clone().unwrap_or_default())),
        },
        ColumnDef {
            title: "Cognome",
            sortable: true,
            searchable: true,
            cell: Arc::new(|u: &User| u.surname.clone().unwrap_or_default().into_any()),
            sort_key: Some(Arc::new(|u| u.surname.clone().unwrap_or_default())),
            search_key: Some(Arc::new(|u| u.surname.clone().unwrap_or_default())),
        },
        ColumnDef {
            title: "Email",
            sortable: true,
            searchable: true,
            cell: Arc::new(|u: &User| u.email.clone().unwrap_or_default().into_any()),
            sort_key: Some(Arc::new(|u| u.email.clone().unwrap_or_default())),
            search_key: Some(Arc::new(|u| u.email.clone().unwrap_or_default())),
        },
        ColumnDef {
            title: "Abilitato",
            sortable: true,
            searchable: false,
            cell: Arc::new(|u: &User| if u.enabled {
                view! { <span class="text-green-500 text-xs font-medium">"S&igrave;"</span> }.into_any()
            } else {
                view! { <span class="text-destructive text-xs font-medium">"No"</span> }.into_any()
            }),
            sort_key: Some(Arc::new(|u| u.enabled.to_string())),
            search_key: None,
        },
    ];

    let actions = {
        let nav = navigate;
        Arc::new(move |u: &User| {
            let uid = u.id.clone().unwrap_or_default();
            let nav = nav.clone();
            view! {
                <button type="button" on:click=move |_| {
                    let _ = nav(&format!("/settings/users/{uid}"), Default::default());
                }
                    class="text-sm text-primary underline hover:no-underline">
                    "Dettaglio"
                </button>
            }.into_any()
        })
    };

    view! {
        <Title text="Utenti - App Gateway"/>

        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-semibold text-foreground mb-1">"Utenti"</h2>
                    <p class="text-sm text-muted-foreground">"Gestisci gli utenti della piattaforma"</p>
                </div>
                <button type="button" on:click=move |_| create_open.set(true)
                    class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors">
                    "Nuovo utente"
                </button>
            </div>

            {move || {
                if loading.get() {
                    view! { <p class="text-sm text-muted-foreground">"Caricamento..."</p> }.into_any()
                } else {
                    view! {
                        <DataTable
                            source=DataTableSource::Client(users.get())
                            columns=columns.clone()
                            initial_page_size=10
                            show_search=true
                            actions=actions.clone()
                        />
                    }.into_any()
                }
            }}
        </div>

        <CreateUserDialog
            open=create_open
            new_name=new_name
            new_surname=new_surname
            new_email=new_email
            new_enabled=new_enabled
            on_create=on_create
        />
    }
}
