use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use crate::modules::base::toast_utils::{toast_error, toast_success, use_toast_ctx};
use crate::modules::base::traits::CrudApi as _;
use crate::modules::siti_osservativi::api::SitiOsservativiApi;
use crate::stores::auth_store::use_auth;
use app_modules::astronomia::siti_osservativi::models::SitoOsservativoCreate;

use valerios_ui_toolkit::button::{Button, ButtonVariant};
use valerios_ui_toolkit::data_table::{ColumnDef, DataTable, DataTableSource};
use valerios_ui_toolkit::dialog::{
    Dialog, DialogClose, DialogContent, DialogFooter, DialogHeader, DialogTitle,
};

#[component]
fn CreateSitoDialog(
    open: RwSignal<bool>,
    new_nome: RwSignal<String>,
    new_lat: RwSignal<f64>,
    new_lng: RwSignal<f64>,
    new_alt: RwSignal<f64>,
    on_create: Callback<()>,
) -> impl IntoView {
    view! {
        <Dialog open=open>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>"Crea nuovo sito osservativo"</DialogTitle>
                </DialogHeader>
                <div class="grid grid-cols-2 gap-4">
                    <div class="col-span-2">
                        <label class="block text-sm font-medium text-foreground mb-1">"Nome"</label>
                        <input type="text" prop:value=new_nome
                            on:input=move |e| new_nome.set(event_target_value(&e))
                            class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-1">"Latitudine"</label>
                        <input type="number" step="any" prop:value=move || new_lng.get().to_string()
                            on:input=move |e| { new_lat.set(event_target_value(&e).parse().unwrap_or(0.0)); }
                            class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-1">"Longitudine"</label>
                        <input type="number" step="any" prop:value=move || new_lng.get().to_string()
                            on:input=move |e| { new_lng.set(event_target_value(&e).parse().unwrap_or(0.0)); }
                            class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-foreground mb-1">"Altitudine (m)"</label>
                        <input type="number" step="any" prop:value=move || new_alt.get().to_string()
                            on:input=move |e| { new_alt.set(event_target_value(&e).parse().unwrap_or(0.0)); }
                            class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                    </div>
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
pub fn SitiList() -> impl IntoView {
    let auth = use_auth();
    let toast = use_toast_ctx();
    let navigate = use_navigate();

    let siti: RwSignal<Vec<app_modules::astronomia::siti_osservativi::models::SitoOsservativo>> =
        RwSignal::new(Vec::new());
    let loading = RwSignal::new(true);
    let create_open = RwSignal::new(false);

    let new_nome = RwSignal::new(String::new());
    let new_lat = RwSignal::new(0.0);
    let new_lng = RwSignal::new(0.0);
    let new_alt = RwSignal::new(0.0);

    let siti_osservativi_api = Arc::new(SitiOsservativiApi::new(auth.get_api_client()));

    {
        let siti_osservativi_api = Arc::clone(&siti_osservativi_api);
        spawn_local(async move {
            match siti_osservativi_api.list().await {
                Ok(list) => siti.set(list),
                Err(e) => toast_error(&toast, &e.to_string()),
            }
            loading.set(false);
        });
    }

    let on_create = Callback::new({
        let toast = toast;
        let siti = siti;
        let create_open = create_open;
        move |_: ()| {
            let body = SitoOsservativoCreate {
                nome: new_nome.get(),
                longitudine: new_lng.get(),
                latitudine: new_lat.get(),
                altitudine: new_alt.get(),
                timezone: None,
            };
            let t = toast;
            spawn_local({
                let siti = siti;
                let siti_osservativi_api = Arc::clone(&siti_osservativi_api);
                async move {
                    match siti_osservativi_api.create(&body).await {
                        Ok(s) => {
                            siti.update(|list| list.push(s));
                            new_nome.set(String::new());
                            new_lat.set(0.0);
                            new_lng.set(0.0);
                            new_alt.set(0.0);
                            toast_success(&t, "Sito osservativo creato");
                            create_open.set(false);
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
            backend_field: None,
            cell: Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    s.nome.clone().into_any()
                },
            ),
            sort_key: Some(Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    s.nome.clone()
                },
            )),
            search_key: Some(Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    s.nome.clone()
                },
            )),
        },
        ColumnDef {
            title: "Latitudine",
            sortable: true,
            searchable: false,
            backend_field: None,
            cell: Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    format!("{:.4}", s.latitudine).into_any()
                },
            ),
            sort_key: Some(Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    s.latitudine.to_string()
                },
            )),
            search_key: None,
        },
        ColumnDef {
            title: "Longitudine",
            sortable: true,
            searchable: false,
            backend_field: None,
            cell: Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    format!("{:.4}", s.longitudine).into_any()
                },
            ),
            sort_key: Some(Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    s.longitudine.to_string()
                },
            )),
            search_key: None,
        },
        ColumnDef {
            title: "Altitudine (m)",
            sortable: true,
            searchable: false,
            backend_field: None,
            cell: Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    format!("{:.1}", s.altitudine).into_any()
                },
            ),
            sort_key: Some(Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    s.altitudine.to_string()
                },
            )),
            search_key: None,
        },
        ColumnDef {
            title: "Timezone",
            sortable: true,
            searchable: false,
            backend_field: None,
            cell: Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    s.timezone.clone().unwrap_or_default().into_any()
                },
            ),
            sort_key: Some(Arc::new(
                |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                    s.timezone.clone().unwrap_or_default()
                },
            )),
            search_key: None,
        },
    ];

    let actions = {
        let nav = navigate;
        Arc::new(
            move |s: &app_modules::astronomia::siti_osservativi::models::SitoOsservativo| {
                let sid = s.id.clone().unwrap_or_default();
                let nav = nav.clone();
                view! {
                    <button
                        type="button"
                        on:click=move |_| {
                            let _ = nav(&format!("/siti_osservativi/{sid}"), Default::default());
                        }
                        class="text-sm text-primary underline hover:no-underline"
                    >
                        "Dettaglio"
                    </button>
                }
                .into_any()
            },
        )
    };

    view! {
        <Title text="Siti Osservativi - App Gateway"/>

        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-semibold text-foreground mb-1">"Siti Osservativi"</h2>
                    <p class="text-sm text-muted-foreground">
                        "Gestisci i siti di osservazione astronomica"
                    </p>
                </div>
                <button
                    type="button"
                    on:click=move |_| create_open.set(true)
                    class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors"
                >
                    "Nuovo sito"
                </button>
            </div>

            {move || {
                if loading.get() {
                    view! { <p class="text-sm text-muted-foreground">"Caricamento..."</p> }
                        .into_any()
                } else {
                    view! {
                        <DataTable
                            source=DataTableSource::Client(siti.get())
                            columns=columns.clone()
                            initial_page_size=10
                            show_search=true
                            actions=actions.clone()
                        />
                    }
                    .into_any()
                }
            }}
        </div>

        <CreateSitoDialog
            open=create_open
            new_nome=new_nome
            new_lat=new_lat
            new_lng=new_lng
            new_alt=new_alt
            on_create=on_create
        />
    }
}
