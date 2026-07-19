#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;
use crate::components::shadcn::icon::Icon;

#[component]
fn ComboboxItem(
    label: &'static str,
    opt_val: &'static str,
    value: RwSignal<String>,
    on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    close: RwSignal<bool>,
) -> impl IntoView {
    let is_selected = move || value.get() == opt_val;
    let handler = move |_| {
        value.set(opt_val.to_string());
        if let Some(ref cb) = on_change {
            cb(opt_val.to_string());
        }
        close.set(false);
    };

    view! {
        <button type="button" role="option"
            on:click=handler
            data-selected=move || if is_selected() { "true" } else { "false" }
            class="relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm hover:bg-accent hover:text-accent-foreground data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground"
        >
            <span class="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
                {move || if is_selected() { Icon::Check.render() } else { ().into_any() }}
            </span>
            <span class="truncate">{label}</span>
        </button>
    }
}

#[component]
fn ComboboxDropdown(
    value: RwSignal<String>,
    on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    open: RwSignal<bool>,
    search: RwSignal<String>,
    options: Vec<(&'static str, &'static str)>,
    empty_text: &'static str,
    search_placeholder: &'static str,
) -> impl IntoView {
    let filtered = Memo::new(move |_| {
        let q = search.get().to_lowercase();
        if q.is_empty() {
            options.clone()
        } else {
            options.iter()
                .filter(|(_, label)| label.to_lowercase().contains(&q))
                .map(|(v, l)| (*v, *l))
                .collect::<Vec<_>>()
        }
    });

    let has_no_results = move || !search.get().is_empty() && filtered.get().is_empty();

    view! {
        <div on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
            on:keydown=move |ev: leptos::ev::KeyboardEvent| { if ev.key() == "Escape" { open.set(false); } }
            class="absolute z-50 top-full left-0 right-0 mt-1 rounded-md border bg-popover text-popover-foreground shadow-md animate-fade-in">
            <div class="flex items-center border-b px-3">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-2 size-4 shrink-0 opacity-50">
                    <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
                </svg>
                <input type="text" placeholder=search_placeholder
                    on:input=move |ev: leptos::ev::Event| search.set(event_target_value(&ev))
                    class="flex h-11 w-full rounded-md bg-transparent py-3 text-sm outline-none placeholder:text-muted-foreground"/>
            </div>
            <div class="max-h-[200px] overflow-y-auto p-1">
                {move || if has_no_results() {
                    view! { <div class="py-6 text-center text-sm text-muted-foreground">{empty_text}</div> }.into_any()
                } else { ().into_any() }}
                <For each=move || filtered.get()
                    key=|(v, _)| *v
                    children=move |(opt_val, label): (&'static str, &'static str)| {
                        let value = value;
                        let on_change = on_change.clone();
                        let open = open;
                        view! {
                            <ComboboxItem label=label opt_val=opt_val
                                value=value on_change=on_change close=open />
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
pub fn Combobox(
    value: RwSignal<String>,
    options: Vec<(&'static str, &'static str)>,
    #[prop(default = "Seleziona...")] placeholder: &'static str,
    #[prop(default = "Cerca...")] search_placeholder: &'static str,
    #[prop(default = "Nessun risultato.")] empty_text: &'static str,
    #[prop(optional)] on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let search = RwSignal::new(String::new());

    let opts = options.clone();
    let selected_label = move || {
        let v = value.get();
        opts.iter()
            .find(|(opt_val, _)| *opt_val == v)
            .map(|(_, label)| *label)
            .unwrap_or(placeholder)
    };

    let chevron = Icon::ChevronDown.render();
    let options_clone = options.clone();


    view! {
        <div class="relative w-full" data-slot="combobox">
            <button type="button" role="combobox"
                on:click=move |_| open.update(|v| *v = !*v)
                class="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2 w-full justify-between">
                <span class="truncate">{selected_label()}</span>
                {chevron}
            </button>

            {move || if open.get() {
                view! {
                    <div on:click=move |_| open.set(false) class="fixed inset-0 z-40" />
                    <ComboboxDropdown value=value on_change=on_change.clone()
                        open=open search=search options=options_clone.clone()
                        empty_text=empty_text search_placeholder=search_placeholder />
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}
