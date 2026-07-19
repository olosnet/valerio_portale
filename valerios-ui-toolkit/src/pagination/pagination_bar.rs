use leptos::prelude::*;
use super::pagination::{Pagination, PaginationContent, PaginationItem, PaginationEllipsis};
use crate::icon::Icon;

fn generate_pages(current: usize, total: usize, siblings: usize) -> Vec<PageItem> {
    if total <= 7 {
        return (0..total).map(|i| PageItem::Page(i)).collect();
    }
    let mut pages = Vec::new();
    let left_end = siblings + 2;
    let right_start = total - siblings - 1;
    if current <= left_end {
        for i in 0..=left_end + 1 { pages.push(PageItem::Page(i)); }
        pages.push(PageItem::Ellipsis);
        pages.push(PageItem::Page(total - 1));
    } else if current >= right_start {
        pages.push(PageItem::Page(0));
        pages.push(PageItem::Ellipsis);
        for i in right_start - 1..total { pages.push(PageItem::Page(i)); }
    } else {
        pages.push(PageItem::Page(0));
        pages.push(PageItem::Ellipsis);
        for i in current - siblings..=current + siblings { pages.push(PageItem::Page(i)); }
        pages.push(PageItem::Ellipsis);
        pages.push(PageItem::Page(total - 1));
    }
    pages
}

#[derive(Clone, PartialEq)]
enum PageItem {
    Page(usize),
    Ellipsis,
}

impl PageItem {
    fn key(&self) -> usize {
        match self {
            Self::Page(i) => *i,
            Self::Ellipsis => 999_999,
        }
    }
}

#[component]
pub fn PaginationBar(
    current_page: RwSignal<usize>,
    total_pages: Signal<usize>,
    #[prop(default = 1)] sibling_count: usize,
) -> impl IntoView {
    let page_items = Memo::new(move |_| generate_pages(current_page.get(), total_pages.get(), sibling_count));

    let icon_left = Icon::ChevronLeft.render();
    let icon_right = Icon::ChevronRight.render();

    let btn_cls = "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-9 px-3";

    // NOTE: >= dentro view! macro rompe parser RSX (il > chiude il tag).
    let is_first_page = move || current_page.get() == 0;
    let is_last_page = move || current_page.get() + 1 >= total_pages.get();

    view! {
        <Pagination>
            <PaginationContent>
                <PaginationItem>
                    <button type="button" disabled=is_first_page
                        on:click=move |_| { let p = current_page.get(); if p > 0 { current_page.set(p - 1); } }
                        class=btn_cls aria-label="Go to previous page">
                        {icon_left}
                        <span class="hidden sm:block">"Precedente"</span>
                    </button>
                </PaginationItem>

                <For each=move || page_items.get()
                    key=|item| item.key()
                    children=move |item: PageItem| {
                        let cp = current_page;
                        match item {
                            PageItem::Page(idx) => {
                                let text = (idx + 1).to_string();
                                let is_active = move || cp.get() == idx;
                                view! {
                                    <PaginationItem>
                                        <button type="button"
                                            aria-current=move || if is_active() { "page" } else { "false" }
                                            on:click=move |_| cp.set(idx)
                                            class=move || if is_active() {
                                                "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-9 w-9 border border-border bg-accent text-accent-foreground"
                                            } else {
                                                "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-9 w-9"
                                            }
                                        >
                                            {text}
                                        </button>
                                    </PaginationItem>
                                }
                            }
                            PageItem::Ellipsis => {
                                view! { <PaginationItem><PaginationEllipsis /></PaginationItem> }
                            }
                        }
                    }
                />

                <PaginationItem>
                    <button type="button" disabled=is_last_page
                        on:click=move |_| { let p = current_page.get(); let t = total_pages.get(); if p + 1 < t { current_page.set(p + 1); } }
                        class=btn_cls aria-label="Go to next page">
                        <span class="hidden sm:block">"Successivo"</span>
                        {icon_right}
                    </button>
                </PaginationItem>
            </PaginationContent>
        </Pagination>
    }
}
