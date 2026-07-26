use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use crate::modules::base::toast_utils::{toast_error, toast_success, use_toast_ctx};
use crate::stores::auth_store::use_auth;

use app_modules::astronomia::oggetti_astronomici::models::{
    OggettoAstronomico, OggettoAstronomicoCreate, TipoOggetto,
};
use valerios_ui_toolkit::button::{Button, ButtonVariant};
use valerios_ui_toolkit::data_table::{
    ColumnDef, DataTable, DataTableResponse, DataTableSource, DataTableState, SortDir,
};
use valerios_ui_toolkit::dialog::{
    Dialog, DialogClose, DialogContent, DialogFooter, DialogHeader, DialogTitle,
};

#[component]
fn CreateOggettoDialog(
    open: RwSignal<bool>,
    new_nome: RwSignal<String>,
    new_tipo_str: RwSignal<String>,
    on_create: Callback<()>,
) -> impl IntoView {
    view! {
        <Dialog open=open>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>"Crea nuovo oggetto astronomico"</DialogTitle>
                </DialogHeader>
                <div class="grid grid-cols-2 gap-4">
                    <div class="col-span-2">
                        <label class="block text-sm font-medium text-foreground mb-1">"Nome comune"</label>
                        <input type="text" prop:value=new_nome
                            on:input=move |e| new_nome.set(event_target_value(&e))
                            class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm"/>
                    </div>
                    <div class="col-span-2">
                        <label class="block text-sm font-medium text-foreground mb-1">"Tipo"</label>
                        <select prop:value=new_tipo_str
                            on:change=move |e| new_tipo_str.set(event_target_value(&e))
                            class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm">
                            <option value="GAL">Galassia</option>
                            <option value="OpC">Ammasso aperto</option>
                            <option value="GCl">Ammasso globulare</option>
                            <option value="Neb">Nebulosa</option>
                            <option value="PN">Nebulosa planetaria</option>
                            <option value="SNR">Resto di supernova</option>
                            <option value="Star">Stella</option>
                        </select>
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
pub fn OggettiList() -> impl IntoView {
    let auth = use_auth();
    let toast = use_toast_ctx();
    let navigate = use_navigate();
    let client = auth.api_client.clone();

    let state = DataTableState {
        data: RwSignal::new(DataTableResponse {
            data: Vec::new(),
            total_count: 0,
        }),
        loading: RwSignal::new(true),
        page: RwSignal::new(0usize),
        page_size: RwSignal::new(10usize),
        sort_field: RwSignal::new(None),
        sort_dir: RwSignal::new(SortDir::None),
        search: RwSignal::new(String::new()),
    };

    let create_open = RwSignal::new(false);
    let new_nome = RwSignal::new(String::new());
    let new_tipo_str = RwSignal::new("GAL".to_string());

    let fetch_data = {
        let client = client.clone();
        let state = state.clone();
        let toast = toast;
        move || {
            let c = client.clone();
            let s = state.clone();
            let t = toast;
            state.loading.set(true);
            spawn_local(async move {
                let sf = s.sort_field.get();
                let sd = match s.sort_dir.get() {
                    SortDir::Asc => Some("asc"),
                    SortDir::Desc => Some("desc"),
                    SortDir::None => None,
                };
                let search_val = s.search.get();
                let search = if search_val.is_empty() {
                    None
                } else {
                    Some(search_val.as_str())
                };
                match crate::modules::oggetti_astronomici::api::list_paginated(
                    &c,
                    s.page.get(),
                    s.page_size.get(),
                    sf.as_deref(),
                    sd,
                    search,
                )
                .await
                {
                    Ok(resp) => s.data.set(resp),
                    Err(e) => toast_error(&t, &e.to_string()),
                }
                s.loading.set(false);
            });
        }
    };

    Effect::new(move |_| {
        let _ = (
            state.page.get(),
            state.page_size.get(),
            state.sort_field.get(),
            state.sort_dir.get(),
            state.search.get(),
        );
        fetch_data();
    });

    let columns = vec![
        ColumnDef {
            title: "Nome comune",
            sortable: true,
            searchable: true,
            backend_field: Some("nome_comune"),
            cell: Arc::new(|o: &OggettoAstronomico| o.nome_comune.clone().into_any()),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| o.nome_comune.clone())),
            search_key: Some(Arc::new(|o: &OggettoAstronomico| o.nome_comune.clone())),
        },
        ColumnDef {
            title: "Costellazione",
            sortable: true,
            searchable: true,
            backend_field: Some("abbr_costellazione"),
            cell: Arc::new(|o: &OggettoAstronomico| o.abbr_costellazione.to_string().into_any()),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| o.abbr_costellazione.to_string())),
            search_key: None,
        },
        ColumnDef {
            title: "Tipo",
            sortable: true,
            searchable: true,
            backend_field: Some("tipo"),
            cell: Arc::new(|o: &OggettoAstronomico| o.tipo.to_string().into_any()),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| o.tipo.to_string())),
            search_key: None,
        },
        ColumnDef {
            title: "AR",
            sortable: true,
            searchable: true,
            backend_field: Some("coord_ar"),
            cell: Arc::new(|o: &OggettoAstronomico| o.coord_ar.clone().into_any()),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| o.coord_ar.clone())),
            search_key: None,
        },
        ColumnDef {
            title: "DEC",
            sortable: true,
            searchable: true,
            backend_field: Some("coord_dec"),
            cell: Arc::new(|o: &OggettoAstronomico| o.coord_dec.clone().into_any()),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| o.coord_dec.clone())),
            search_key: None,
        },
        ColumnDef {
            title: "Mag.",
            sortable: true,
            searchable: false,
            backend_field: Some("mag_apparente"),
            cell: Arc::new(|o: &OggettoAstronomico| {
                o.mag_apparente
                    .map(|m| format!("{:.1}", m))
                    .unwrap_or_else(|| "—".to_string())
                    .into_any()
            }),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| {
                o.mag_apparente.map(|m| format!("{:0>6.1}", m)).unwrap_or_default()
            })),
            search_key: None,
        },
        ColumnDef {
            title: "Cataloghi",
            sortable: false,
            searchable: true,
            backend_field: None,
            cell: Arc::new(|o: &OggettoAstronomico| {
                o.cataloghi.iter().map(|c| c.extended.clone()).collect::<Vec<_>>().join(", ").into_any()
            }),
            sort_key: None,
            search_key: Some(Arc::new(|o: &OggettoAstronomico| {
                o.cataloghi.iter().map(|c| c.extended.clone()).collect::<Vec<_>>().join(" ")
            })),
        },
    ];

    let actions = {
        let nav = navigate;
        Arc::new(move |o: &OggettoAstronomico| {
            let oid = o.id.clone().unwrap_or_default();
            let nav = nav.clone();
            view! {
                <button
                    type="button"
                    on:click=move |_| { let _ = nav(&format!("/oggetti_astronomici/{oid}"), Default::default()); }
                    class="text-sm text-primary underline hover:no-underline"
                >
                    "Dettaglio"
                </button>
            }
            .into_any()
        })
    };

    let on_create = Callback::new({
        let client = client.clone();
        let toast = toast;
        let state = state.clone();
        let create_open = create_open;
        move |_: ()| {
            let tipo = match new_tipo_str.get().as_str() {
                "OpC" => TipoOggetto::AmmassoAperto,
                "GCl" => TipoOggetto::AmmassoGlobulare,
                "Neb" => TipoOggetto::Nebulosa,
                "PN" => TipoOggetto::NebulosaPlanetaria,
                "SNR" => TipoOggetto::RestoSupernova,
                "Star" => TipoOggetto::Stella,
                _ => TipoOggetto::Galassia,
            };
            let body = OggettoAstronomicoCreate {
                nome_comune: new_nome.get(),
                tipo,
                ..Default::default()
            };
            let t = toast;
            spawn_local({
                let client = client.clone();
                let state = state.clone();
                async move {
                    match crate::modules::oggetti_astronomici::api::create_oggetto(&client, &body).await {
                        Ok(_) => {
                            new_nome.set(String::new());
                            new_tipo_str.set("GAL".to_string());
                            toast_success(&t, "Oggetto astronomico creato");
                            create_open.set(false);
                            state.page.set(0);
                        }
                        Err(e) => toast_error(&t, &e.to_string()),
                    }
                }
            });
        }
    });

    view! {
        <Title text="Oggetti Astronomici - App Gateway"/>

        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-semibold text-foreground mb-1">"Oggetti Astronomici"</h2>
                    <p class="text-sm text-muted-foreground">"Catalogo degli oggetti celesti"</p>
                </div>
                <button
                    type="button"
                    on:click=move |_| create_open.set(true)
                    class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors"
                >
                    "Nuovo oggetto"
                </button>
            </div>

            <DataTable
                source=DataTableSource::Server(state)
                columns=columns.clone()
                initial_page_size=10
                show_search=true
                actions=actions.clone()
            />
        </div>

        <CreateOggettoDialog
            open=create_open
            new_nome=new_nome
            new_tipo_str=new_tipo_str
            on_create=on_create
        />
    }
}
