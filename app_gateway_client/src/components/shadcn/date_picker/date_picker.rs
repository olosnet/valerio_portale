#![allow(dead_code)]
use std::sync::Arc;
use chrono::NaiveDate;
use leptos::prelude::*;
use crate::components::shadcn::shared::{OverlayProvider, use_overlay};
use crate::components::shadcn::calendar::{Calendar, Locale, locale_en};
use crate::components::shadcn::icon::Icon;

fn icon_calendar() -> leptos::prelude::AnyView { Icon::LayoutDashboard.render() }

#[component]
pub fn DatePicker(
    date: RwSignal<Option<NaiveDate>>,
    #[prop(default = "Pick a date")] placeholder: &'static str,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] locale: Option<Locale>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let extra = class.unwrap_or("");
    let loc = locale.unwrap_or_else(locale_en);

    let display = {
        let loc = loc.clone();
        move || {
            match date.get() {
                Some(d) => loc.format_date(d),
                None => placeholder.to_string(),
            }
        }
    };

    let on_select: Arc<dyn Fn(Option<NaiveDate>) + Send + Sync> = Arc::new({
        let date = date;
        let open = open;
        move |d| {
            date.set(d);
            open.set(false);
        }
    });

    view! {
        <div data-slot="date-picker" class=extra>
            <OverlayProvider open=open>
                <div class="relative w-full">
                    <button type="button" role="combobox"
                        on:click=move |_| open.update(|v| *v = !*v)
                        class="inline-flex items-center justify-start gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2 w-full">
                        {icon_calendar()}
                        <span class=move || if date.get().is_some() { "" } else { "text-muted-foreground" }>
                            {display()}
                        </span>
                    </button>

                    {move || if open.get() {
                        view! {
                            <div on:click=move |_| open.set(false) class="fixed inset-0 z-40" />
                            <div on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
                                class="absolute z-50 top-full left-0 right-0 mt-1">
                                <Calendar selected=date on_select=on_select.clone()
                                    locale=loc.clone() />
                            </div>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            </OverlayProvider>
        </div>
    }
}
