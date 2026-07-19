use leptos::prelude::*;

fn icon_chevron_left() -> leptos::prelude::AnyView { crate::icon::Icon::ChevronLeft.render() }
fn icon_chevron_right() -> leptos::prelude::AnyView { crate::icon::Icon::ChevronRight.render() }

#[component]
pub fn DataTablePagination(
    page: RwSignal<usize>,
    page_size: RwSignal<usize>,
    total_count: Signal<usize>,
    total_pages: Signal<usize>,
    page_size_options: Vec<usize>,
) -> impl IntoView {
    let btn_class = "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-8 w-8";

    // NOTE: >= dentro view! macro rompe parser RSX (il > chiude il tag).
    // Estrarre closure con > o >= fuori dal view!.
    let is_first_page = move || page.get() == 0;
    let is_last_page = move || page.get() + 1 >= total_pages.get();

    view! {
        <div data-slot="data-table-pagination" class="flex items-center justify-between px-2">
            <div class="text-sm text-muted-foreground">
                {move || {
                    let tc = total_count.get();
                    if tc == 0 {
                        "Nessun risultato.".to_string()
                    } else {
                        let p = page.get();
                        let ps = page_size.get();
                        let start = p * ps + 1;
                        let end = ((p * ps + ps).min(tc));
                        format!("Mostrando {} a {} di {}", start, end, tc)
                    }
                }}
            </div>

            <div class="flex items-center gap-4">
                <div class="flex items-center gap-2">
                    <span class="text-sm font-medium">"Righe per pagina"</span>
                    <select
                        prop:value=move || page_size.get().to_string()
                        on:change=move |ev: leptos::ev::Event| {
                            let val: usize = event_target_value(&ev).parse().unwrap_or(10);
                            page_size.set(val);
                            page.set(0);
                        }
                        class="flex h-8 w-[70px] rounded-md border border-input bg-background px-2 py-1 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                    >
                        {move || page_size_options.iter().map(|opt| {
                            let sel = *opt == page_size.get();
                            view! {
                                <option value=opt.to_string() selected=sel>
                                    {opt.to_string()}
                                </option>
                            }
                        }).collect_view()}
                    </select>
                </div>

                <div class="text-sm font-medium">
                    {move || {
                        let tp = total_pages.get();
                        if tp == 0 { "Pagina 0 di 0".to_string() } else { format!("Pagina {} di {}", page.get() + 1, tp) }
                    }}
                </div>

                <div class="flex items-center gap-1">
                    <button type="button"
                        disabled=is_first_page
                        on:click=move |_| page.set(0)
                        class=btn_class aria-label="Prima pagina">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4"><path d="m11 17-5-5 5-5"/><path d="m18 17-5-5 5-5"/></svg>
                    </button>
                    <button type="button"
                        disabled=is_first_page
                        on:click=move |_| { let p = page.get(); if p > 0 { page.set(p - 1); } }
                        class=btn_class aria-label="Pagina precedente">
                        {icon_chevron_left()}
                    </button>
                    <button type="button"
                        disabled=is_last_page
                        on:click=move |_| { let p = page.get(); let t = total_pages.get(); if p + 1 < t { page.set(p + 1); } }
                        class=btn_class aria-label="Pagina successiva">
                        {icon_chevron_right()}
                    </button>
                    <button type="button"
                        disabled=is_last_page
                        on:click=move |_| { let t = total_pages.get(); page.set(t.saturating_sub(1)); }
                        class=btn_class aria-label="Ultima pagina">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4"><path d="m7 7 5 5-5 5"/><path d="m13 7 5 5-5 5"/></svg>
                    </button>
                </div>
            </div>
        </div>
    }
}
