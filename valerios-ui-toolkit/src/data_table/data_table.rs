use std::sync::Arc;
use leptos::prelude::*;
use super::types::*;
use super::data_table_pagination::DataTablePagination;
use crate::icon::Icon;

fn icon_search() -> leptos::prelude::AnyView { Icon::Search.render() }
fn icon_up() -> leptos::prelude::AnyView { Icon::ChevronUp.render() }
fn icon_down() -> leptos::prelude::AnyView { Icon::ChevronDown.render() }
fn icon_updown() -> leptos::prelude::AnyView { Icon::ChevronsUpDown.render() }

fn sort_icon_state(sort_field: Option<String>, sort_dir: SortDir, col_title: &str) -> AnyView {
    let active = sort_field.as_deref() == Some(col_title);
    if !active { return icon_updown(); }
    match sort_dir {
        SortDir::Asc => icon_up(),
        SortDir::Desc => icon_down(),
        SortDir::None => icon_updown(),
    }
}

#[component]
fn DataTableRowActions<T: Clone + 'static>(
    actions: Option<Arc<dyn Fn(&T) -> AnyView + Send + Sync>>,
    item: T,
    width: &'static str,
) -> impl IntoView {
    if let Some(act_fn) = actions.as_ref() {
        view! {
            <td class=format!("p-4 align-middle text-right {}", width)>
                {act_fn(&item)}
            </td>
        }.into_any()
    } else { ().into_any() }
}

#[component]
pub fn DataTable<T: Clone + Send + Sync + 'static>(
    columns: Vec<ColumnDef<T>>,
    source: DataTableSource<T>,
    #[prop(default = 10)] initial_page_size: usize,
    #[prop(optional)] page_size_options: Option<Vec<usize>>,
    #[prop(default = true)] show_search: bool,
    #[prop(optional)] actions: Option<Arc<dyn Fn(&T) -> AnyView + Send + Sync>>,
    #[prop(default = "w-[80px]")] actions_width: &'static str,
    #[prop(default = "Azioni")] actions_title: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let page = RwSignal::new(0usize);
    let page_size = RwSignal::new(initial_page_size);
    let sort_field: RwSignal<Option<String>> = RwSignal::new(None);
    let sort_dir: RwSignal<SortDir> = RwSignal::new(SortDir::None);
    let search: RwSignal<String> = RwSignal::new(String::new());
    let extra = class.unwrap_or("");

    let memo_columns = columns.clone();
    let result = Memo::new(move |_| {
        let mut items = match &source {
            DataTableSource::Client(data) => data.clone(),
        };
        let q = search.get().to_lowercase();
        if !q.is_empty() {
            items.retain(|item| {
                memo_columns.iter()
                    .filter(|c| c.searchable)
                    .filter_map(|c| c.search_key.as_ref())
                    .any(|sk| sk(item).to_lowercase().contains(&q))
            });
        }
        let sf = sort_field.get();
        let sd = sort_dir.get();
        if let Some(field) = sf.as_ref() {
            if sd != SortDir::None {
                if let Some(col) = memo_columns.iter().find(|c| c.title == *field) {
                    if let Some(sk) = col.sort_key.as_ref() {
                        items.sort_by(|a, b| sk(a).cmp(&sk(b)));
                        if sd == SortDir::Desc { items.reverse(); }
                    }
                }
            }
        }
        let total = items.len();
        let ps = page_size.get();
        let p = page.get();
        let start = p * ps;
        let end = (start + ps).min(total);
        let data = if start < total { items[start..end].to_vec() } else { vec![] };
        DataTableResponse { data, total_count: total }
    });

    let total_pages = Signal::derive(move || {
        let total = result.get().total_count;
        let ps = page_size.get();
        if total == 0 { 1 } else { (total + ps - 1) / ps }
    });

    let has_actions = actions.is_some();
    let actions_cloned = actions.clone();
    let columns_render = columns.clone();

    let toggle_sort = {
        let sort_field = sort_field;
        let sort_dir = sort_dir;
        move |col_title: &'static str| {
            let current = sort_field.get();
            if current.as_deref() != Some(col_title) {
                sort_field.set(Some(col_title.to_string()));
                sort_dir.set(SortDir::Asc);
            } else {
                match sort_dir.get() {
                    SortDir::Asc => sort_dir.set(SortDir::Desc),
                    SortDir::Desc => { sort_field.set(None); sort_dir.set(SortDir::None); }
                    SortDir::None => { sort_field.set(Some(col_title.to_string())); sort_dir.set(SortDir::Asc); }
                }
            }
        }
    };

    let search_input = {
        let search = search;
        let page = page;
        move |ev: leptos::ev::Event| {
            search.set(event_target_value(&ev));
            page.set(0);
        }
    };

    view! {
        <div data-slot="data-table" class=format!("space-y-4 {}", extra)>
            {move || if show_search {
                view! {
                    <div class="flex items-center gap-2">
                        {icon_search()}
                        <input type="text" placeholder="Cerca..."
                            prop:value=move || search.get()
                            on:input=move |ev: leptos::ev::Event| search.set(event_target_value(&ev))
                            class="flex h-9 w-full max-w-sm rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                        />
                    </div>
                }.into_any()
            } else { ().into_any() }}

            <div class="rounded-md border overflow-hidden">
                <table data-slot="table" class="w-full caption-bottom text-sm">
                    <thead class="[&_tr]:border-b">
                        <tr class="border-b transition-colors hover:bg-muted/50">
                            {move || {
                                let mut views: Vec<AnyView> = Vec::new();
                                let sf = sort_field;
                                let sd = sort_dir;
                                let toggler = toggle_sort;
                                for col in &columns_render {
                                    let col_title = col.title;
                                    views.push(if col.sortable {
                                        let h = toggler;
                                        view! {
                                            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">
                                                <button type="button"
                                                    on:click=move |_| h(col_title)
                                                    class="inline-flex items-center gap-1 hover:text-foreground font-medium">
                                                    {(col.title)}
                                                    {sort_icon_state(sf.get(), sd.get(), col_title)}
                                                </button>
                                            </th>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">
                                                {(col.title)}
                                            </th>
                                        }.into_any()
                                    });
                                }
                                if has_actions {
                                    views.push(view! {
                                        <th class=format!("h-12 px-4 text-right align-middle font-medium text-muted-foreground {}", actions_width)>
                                            <span class="sr-only">{actions_title}</span>
                                        </th>
                                    }.into_any());
                                }
                                views.into_iter().collect_view()
                            }}
                        </tr>
                    </thead>
                    <tbody class="[&_tr:last-child]:border-0">
                        {move || {
                            let res = result.get();
                            if res.data.is_empty() {
                                view! {
                                    <tr class="border-b transition-colors hover:bg-muted/50">
                                        <td class="p-4 align-middle text-center text-muted-foreground h-24">"Nessun risultato."</td>
                                    </tr>
                                }.into_any()
                            } else {
                                let act_opt = actions_cloned.clone();
                                let w = actions_width;
                                let rows: Vec<AnyView> = res.data.iter().map(|item| {
                                    let cells: Vec<AnyView> = columns.iter().map(|col| {
                                        view! { <td class="p-4 align-middle">{(col.cell)(item)}</td> }.into_any()
                                    }).collect();
                                    let cell_act = if let Some(act_fn) = act_opt.as_ref() {
                                        view! { <td class=format!("p-4 align-middle text-right {}", w)>{act_fn(item)}</td> }.into_any()
                                    } else { ().into_any() };
                                    view! {
                                        <tr class="border-b transition-colors hover:bg-muted/50">
                                            {cells.into_iter().collect_view()}
                                            {cell_act}
                                        </tr>
                                    }.into_any()
                                }).collect();
                                view! { {rows.into_iter().collect_view()} }.into_any()
                            }
                        }}
                    </tbody>
                </table>
            </div>

            <DataTablePagination
                page=page
                page_size=page_size
                total_count=Signal::derive(move || result.get().total_count)
                total_pages=total_pages
                page_size_options=page_size_options.unwrap_or_else(|| vec![10, 20, 30, 40, 50])
            />
        </div>
    }
}
