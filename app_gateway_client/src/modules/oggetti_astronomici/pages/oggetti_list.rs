use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use serde::Deserialize;
use serde_json::json;
use wasm_bindgen::prelude::*;

use crate::modules::base::toast_utils::{toast_error, toast_success, use_toast_ctx};
use crate::stores::auth_store::use_auth;

use app_modules::astronomia::oggetti_astronomici::models::{
    OggettoAstronomico, OggettoAstronomicoCreate,
};
use app_modules::astronomia::oggetti_astronomici::models::TipoOggetto;

fn parse_tipo(s: &str) -> TipoOggetto {
    serde_json::from_str(&format!("\"{}\"", s)).unwrap_or_default()
}
use app_modules::astronomia::common::helpers::{format_ar, format_dec};
use valerios_ui_toolkit::button::{Button, ButtonVariant};
use valerios_ui_toolkit::data_table::{
    ColumnDef, DataTable, DataTableResponse, DataTableSource, DataTableState, SortDir,
};
use valerios_ui_toolkit::dialog::{
    Dialog, DialogClose, DialogContent, DialogFooter, DialogHeader, DialogTitle,
};

#[derive(Deserialize, Clone)]
struct EnumValue { name: String, value: String }

#[derive(Deserialize)]
struct StaticsResp { tipo_oggetto: Vec<EnumValue>, costellazioni: Vec<EnumValue> }

#[component]
fn CreateOggettoDialog(
    open: RwSignal<bool>,
    new_nome: RwSignal<String>,
    new_tipo_str: RwSignal<String>,
    tipi: RwSignal<Vec<EnumValue>>,
    on_create: Callback<()>,
) -> impl IntoView {
    view! {
        <Dialog open=open>
            <DialogContent>
                <DialogHeader><DialogTitle>"Crea nuovo oggetto astronomico"</DialogTitle></DialogHeader>
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
                            {move || tipi.get().iter().map(|t| view! { <option value={t.value.clone()}>{t.name.clone()}</option> }.into_any()).collect::<Vec<AnyView>>()}
                        </select>
                    </div>
                </div>
                <DialogFooter>
                    <Button on_click=Arc::new(move || on_create.run(()))>"Crea"</Button>
                    <DialogClose><Button variant=ButtonVariant::Outline>"Annulla"</Button></DialogClose>
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
        data: RwSignal::new(DataTableResponse { data: Vec::new(), total_count: 0 }),
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

    let filter_tipo = RwSignal::new(String::new());
    let filter_costellazione = RwSignal::new(String::new());
    let filter_mag_min = RwSignal::new(String::new());
    let filter_mag_max = RwSignal::new(String::new());

    let all_tipi: RwSignal<Vec<EnumValue>> = RwSignal::new(Vec::new());
    let all_costellazioni: RwSignal<Vec<EnumValue>> = RwSignal::new(Vec::new());

    {
        let client = client.clone();
        spawn_local(async move {
            if let Ok(json) = client.request("GET", "/statics", None).await {
                if let Ok(resp) = serde_json::from_str::<StaticsResp>(&json) {
                    all_tipi.set(resp.tipo_oggetto);
                    all_costellazioni.set(resp.costellazioni);
                }
            }
        });
    }

    let fetch_data = Rc::new({
        let client = client.clone();
        let state = state.clone();
        let toast = toast;
        move || {
            let c = client.clone();
            let s = state.clone();
            let t = toast;
            state.loading.set(true);
            spawn_local(async move {
                let search_val = s.search.get();
                let search = if search_val.is_empty() { None } else { Some(search_val.as_str()) };
                let mut filter_parts: Vec<serde_json::Value> = Vec::new();
                let flt_tipo = filter_tipo.get();
                if !flt_tipo.is_empty() { filter_parts.push(json!({"field": "tipo", "op": "eq", "value": flt_tipo})); }
                let flt_cost = filter_costellazione.get();
                if !flt_cost.is_empty() { filter_parts.push(json!({"field": "abbr_costellazione", "op": "eq", "value": flt_cost})); }
                if let Ok(val) = filter_mag_min.get().parse::<f64>() { filter_parts.push(json!({"field": "mag_apparente", "op": "gte", "value": val})); }
                if let Ok(val) = filter_mag_max.get().parse::<f64>() { filter_parts.push(json!({"field": "mag_apparente", "op": "lte", "value": val})); }
                let filters = if filter_parts.len() > 1 {
                    Some(json!({"and": filter_parts}).to_string())
                } else if filter_parts.len() == 1 {
                    Some(filter_parts.into_iter().next().unwrap().to_string())
                } else { None };

                match crate::modules::oggetti_astronomici::api::list_paginated(
                    &c, s.page.get(), s.page_size.get(),
                    s.sort_field.get().as_deref(),
                    match s.sort_dir.get() { SortDir::Asc => Some("asc"), SortDir::Desc => Some("desc"), SortDir::None => None },
                    search, filters.as_deref(),
                ).await {
                    Ok(resp) => s.data.set(resp),
                    Err(e) => toast_error(&t, &e.to_string()),
                }
                s.loading.set(false);
            });
        }
    });

    let debounce_timer: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

    let cancel_timer = {
        let timer = debounce_timer.clone();
        Rc::new(move || {
            if let Some(id) = timer.borrow_mut().take() {
                let _ = web_sys::window().unwrap().clear_timeout_with_handle(id);
            }
        })
    };

    let start_timer = {
        let timer = debounce_timer.clone();
        let fetch = fetch_data.clone();
        let cancel = cancel_timer.clone();
        Rc::new(move || {
            cancel();
            let f = fetch.clone();
            let cb = Closure::wrap(Box::new(move || f()) as Box<dyn FnMut()>);
            let id = web_sys::window().unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(), 300,
                ).unwrap();
            *timer.borrow_mut() = Some(id);
            cb.forget();
        })
    };

    let fetch_immediate = {
        let fetch = fetch_data.clone();
        let cancel = cancel_timer.clone();
        Rc::new(move || {
            cancel();
            fetch();
        })
    };

    {
        let fetch = fetch_data.clone();
        Effect::new(move |_| {
            let _ = (state.page.get(), state.page_size.get(),
                     state.sort_field.get(), state.sort_dir.get());
            fetch();
        });
    }

    let columns = vec![
        ColumnDef {
            title: "Nome comune", sortable: true, searchable: true, backend_field: Some("nome_comune"),
            cell: Arc::new(|o: &OggettoAstronomico| o.nome_comune.clone().into_any()),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| o.nome_comune.clone())),
            search_key: Some(Arc::new(|o: &OggettoAstronomico| o.nome_comune.clone())),
        },
        ColumnDef {
            title: "Costellazione", sortable: true, searchable: true, backend_field: Some("abbr_costellazione"),
            cell: Arc::new(|o: &OggettoAstronomico| o.abbr_costellazione.to_string().into_any()),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| o.abbr_costellazione.to_string())),
            search_key: None,
        },
        ColumnDef {
            title: "Tipo", sortable: true, searchable: true, backend_field: Some("tipo"),
            cell: Arc::new(|o: &OggettoAstronomico| o.tipo.to_string().into_any()),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| o.tipo.to_string())),
            search_key: None,
        },
        ColumnDef {
            title: "AR", sortable: true, searchable: true, backend_field: Some("coord_ar"),
            cell: Arc::new(|o: &OggettoAstronomico| format_ar(&o.coord_ar).into_any()),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| o.coord_ar.clone())),
            search_key: None,
        },
        ColumnDef {
            title: "DEC", sortable: true, searchable: true, backend_field: Some("coord_dec"),
            cell: Arc::new(|o: &OggettoAstronomico| format_dec(&o.coord_dec).into_any()),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| o.coord_dec.clone())),
            search_key: None,
        },
        ColumnDef {
            title: "Mag.", sortable: true, searchable: false, backend_field: Some("mag_apparente"),
            cell: Arc::new(|o: &OggettoAstronomico| {
                o.mag_apparente.map(|m| format!("{:.1}", m)).unwrap_or_else(|| "—".to_string()).into_any()
            }),
            sort_key: Some(Arc::new(|o: &OggettoAstronomico| {
                o.mag_apparente.map(|m| format!("{:0>6.1}", m)).unwrap_or_default()
            })),
            search_key: None,
        },
        ColumnDef {
            title: "Cataloghi", sortable: false, searchable: true, backend_field: None,
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
                <button type="button"
                    on:click=move |_| { let _ = nav(&format!("/oggetti_astronomici/{oid}"), Default::default()); }
                    class="text-sm text-primary underline hover:no-underline">"Dettaglio"</button>
            }.into_any()
        })
    };

    let on_create = Callback::new({
        let client = client.clone();
        let toast = toast;
        let state = state.clone();
        let create_open = create_open;
        move |_: ()| {
            let tipo = parse_tipo(&new_tipo_str.get());
            let body = OggettoAstronomicoCreate { nome_comune: new_nome.get(), tipo, ..Default::default() };
            spawn_local({
                let client = client.clone();
                let state = state.clone();
                async move {
                    match crate::modules::oggetti_astronomici::api::create_oggetto(&client, &body).await {
                        Ok(_) => {
                            new_nome.set(String::new()); new_tipo_str.set("GAL".to_string());
                            toast_success(&toast, "Oggetto astronomico creato");
                            create_open.set(false); state.page.set(0);
                        }
                        Err(e) => toast_error(&toast, &e.to_string()),
                    }
                }
            });
        }
    });

    let st = start_timer.clone();
    let fi = fetch_immediate.clone();
    let fi2 = fetch_immediate.clone();

    view! {
        <Title text="Oggetti Astronomici - App Gateway"/>
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-semibold text-foreground mb-1">"Oggetti Astronomici"</h2>
                    <p class="text-sm text-muted-foreground">"Catalogo degli oggetti celesti"</p>
                </div>
                <button type="button"
                    on:click=move |_| create_open.set(true)
                    class="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors">"Nuovo oggetto"</button>
            </div>

            <div class="flex items-end gap-3 flex-wrap">
                <div class="flex-1 min-w-[200px]">
                    <label class="block text-xs font-medium text-muted-foreground mb-1">"Cerca"</label>
                    <input type="text" placeholder="Cerca oggetti..."
                        prop:value=move || state.search.get()
                        on:input=move |ev: leptos::ev::Event| {
                            state.search.set(event_target_value(&ev));
                            state.page.set(0);
                            st();
                        }
                        class="h-9 w-full px-3 rounded-md border border-input bg-background text-foreground text-sm"
                    />
                </div>
                <div>
                    <label class="block text-xs font-medium text-muted-foreground mb-1">"Tipo"</label>
                    <select prop:value=filter_tipo
                        on:change=move |e| { filter_tipo.set(event_target_value(&e)); state.page.set(0); fi(); }
                        class="h-9 px-3 rounded-md border border-input bg-background text-foreground text-sm min-w-[130px]">
                        <option value="">Tutti</option>
                        {move || all_tipi.get().iter().map(|t| view! { <option value={t.value.clone()}>{t.name.clone()}</option> }.into_any()).collect::<Vec<AnyView>>()}
                    </select>
                </div>
                <div>
                    <label class="block text-xs font-medium text-muted-foreground mb-1">"Costellazione"</label>
                    <select prop:value=filter_costellazione
                        on:change=move |e| { filter_costellazione.set(event_target_value(&e)); state.page.set(0); fi2(); }
                        class="h-9 px-3 rounded-md border border-input bg-background text-foreground text-sm min-w-[130px]">
                        <option value="">Tutte</option>
                        {move || all_costellazioni.get().iter().map(|c| view! { <option value={c.value.clone()}>{c.name.clone()}</option> }.into_any()).collect::<Vec<AnyView>>()}
                    </select>
                </div>
                <div>
                    <label class="block text-xs font-medium text-muted-foreground mb-1">"Mag min"</label>
                    <input type="number" step="any"
                        prop:value=filter_mag_min
                        on:input=move |e| { filter_mag_min.set(event_target_value(&e)); state.page.set(0); }
                        class="h-9 w-20 px-3 rounded-md border border-input bg-background text-foreground text-sm"
                    />
                </div>
                <div>
                    <label class="block text-xs font-medium text-muted-foreground mb-1">"Mag max"</label>
                    <input type="number" step="any"
                        prop:value=filter_mag_max
                        on:input=move |e| { filter_mag_max.set(event_target_value(&e)); state.page.set(0); }
                        class="h-9 w-20 px-3 rounded-md border border-input bg-background text-foreground text-sm"
                    />
                </div>
            </div>

            <DataTable
                source=DataTableSource::Server(state)
                columns=columns.clone()
                initial_page_size=10
                show_search=false
                actions=actions.clone()
            />
        </div>

        <CreateOggettoDialog
            open=create_open
            new_nome=new_nome
            new_tipo_str=new_tipo_str
            tipi=all_tipi
            on_create=on_create
        />
    }
}
