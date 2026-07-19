use std::sync::Arc;
use chrono::{NaiveDate, Datelike};
use leptos::prelude::*;
use super::locale::{locale_en, Locale};
use super::utils::{today, prev_month_date, next_month_date, format_month_year, weekday_headers, generate_month_days, DayInfo};
use crate::icon::Icon;

#[component]
pub fn Calendar(
    selected: RwSignal<Option<NaiveDate>>,
    #[prop(optional)] on_select: Option<Arc<dyn Fn(Option<NaiveDate>) + Send + Sync>>,
    #[prop(optional)] from_date: Option<NaiveDate>,
    #[prop(optional)] to_date: Option<NaiveDate>,
    #[prop(default = true)] show_outside_days: bool,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] locale: Option<Locale>,
) -> impl IntoView {
    let loc = locale.unwrap_or_else(locale_en);
    let today_date = today();
    let view_month: RwSignal<(i32, u32)> = RwSignal::new((
        selected.get().map(|d| d.year()).unwrap_or(today_date.year()),
        selected.get().map(|d| d.month()).unwrap_or(today_date.month()),
    ));

    let prev_click = move |_| {
        let (y, m) = view_month.get();
        view_month.set(prev_month_date(y, m));
    };

    let next_click = move |_| {
        let (y, m) = view_month.get();
        view_month.set(next_month_date(y, m));
    };

    let first_day = loc.first_day_of_week;

    let days = Memo::new(move |_| {
        let (y, m) = view_month.get();
        generate_month_days(y, m, today_date, first_day)
    });

    let header_text = {
        let loc = loc.clone();
        move || {
            let (y, m) = view_month.get();
            format_month_year(y, m, &loc)
        }
    };

    let wk_headers = weekday_headers(&loc);

    let extra = class.unwrap_or("");
    let wrapper_class = move || format!("p-3 rounded-lg border bg-card text-card-foreground shadow-sm {}", extra);

    let chevron_left = Icon::ChevronLeft.render();
    let chevron_right = Icon::ChevronRight.render();

    view! {
        <div data-slot="calendar" role="application" class=wrapper_class()>
            <div class="flex items-center justify-between mb-2">
                <button type="button" on:click=prev_click
                    class="inline-flex items-center justify-center rounded-md text-sm ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground h-7 w-7 bg-transparent p-0 opacity-50 hover:opacity-100">
                    {chevron_left}
                </button>
                <h2 class="text-sm font-medium">{header_text()}</h2>
                <button type="button" on:click=next_click
                    class="inline-flex items-center justify-center rounded-md text-sm ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground h-7 w-7 bg-transparent p-0 opacity-50 hover:opacity-100">
                    {chevron_right}
                </button>
            </div>

            <div class="grid grid-cols-7 gap-0 mb-1">
                {wk_headers.into_iter().map(|wd| {
                    view! {
                        <div class="h-8 w-full flex items-center justify-center text-xs font-medium text-muted-foreground">
                            {wd}
                        </div>
                    }
                }).collect_view()}
            </div>

            <div class="grid grid-cols-7 gap-0">
                <For each=move || days.get()
                    key=|d| format!("{}-{}-{}", d.date.year(), d.date.month(), d.date.day())
                    children=move |day: DayInfo| {
                        let day = day;
                        let on_select = on_select.clone();
                        let is_selected = move || selected.get().map_or(false, |s| s == day.date);
                        let is_disabled = move || {
                            if !show_outside_days && day.is_outside { return true; }
                            if let Some(from) = from_date { if day.date < from { return true; } }
                            if let Some(to) = to_date { if day.date > to { return true; } }
                            false
                        };
                        let is_outside = day.is_outside;

                        let handle_click = move |_| {
                            if !is_disabled() && !is_outside {
                                let new = Some(day.date);
                                selected.set(new);
                                if let Some(ref cb) = on_select { cb(new); }
                            }
                        };

                        let day_num = day.date.day().to_string();

                        let cell_cls = move || {
                            let mut cls = "h-8 w-full p-0 text-sm font-normal ring-offset-background transition-colors rounded-md inline-flex items-center justify-center focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2".to_string();
                            if is_outside && show_outside_days { cls.push_str(" text-muted-foreground opacity-50"); }
                            if is_outside && !show_outside_days { cls.push_str(" hidden"); }
                            if !is_outside { cls.push_str(" hover:bg-accent hover:text-accent-foreground"); }
                            if is_selected() { cls.push_str(" bg-primary text-primary-foreground hover:bg-primary hover:text-primary-foreground"); }
                            if day.is_today && !is_selected() { cls.push_str(" bg-accent text-accent-foreground"); }
                            if is_disabled() { cls.push_str(" opacity-30 cursor-not-allowed"); }
                            cls
                        };

                        view! {
                            <button type="button" role="gridcell"
                                data-selected=move || if is_selected() { "true" } else { "false" }
                                data-today=if day.is_today { "true" } else { "false" }
                                data-outside=if day.is_outside { "true" } else { "false" }
                                disabled=is_disabled()
                                on:click=handle_click
                                class=cell_cls()
                            >
                                {day_num}
                            </button>
                        }
                    }
                />
            </div>
        </div>
    }
}
