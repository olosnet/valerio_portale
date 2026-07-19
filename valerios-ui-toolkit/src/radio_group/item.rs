use leptos::prelude::*;
use super::radio_group::use_radio_group;

#[component]
pub fn RadioGroupItem(
    value: &'static str,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] id: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_radio_group();
    let extra = class.unwrap_or("");
    let cls = move || {
        let selected = ctx.value.get() == value;
        format!(
            "aspect-square h-4 w-4 rounded-full border border-primary text-primary ring-offset-background focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground {}",
            extra,
        )
    };

    let handle_click = move |_| {
        ctx.value.set(value.to_string());
        if let Some(ref cb) = ctx.on_change {
            cb(value.to_string());
        }
    };

    view! {
        <button
            id=id
            type="button"
            role="radio"
            aria-checked=move || if ctx.value.get() == value { "true" } else { "false" }
            data-state=move || if ctx.value.get() == value { "checked" } else { "unchecked" }
            on:click=handle_click
            class=cls()
        >
            {move || if ctx.value.get() == value {
                view! {
                    <span class="flex items-center justify-center">
                        <svg xmlns="http://www.w3.org/2000/svg" width="8" height="8" viewBox="0 0 24 24" fill="currentColor" class="size-2.5">
                            <circle cx="12" cy="12" r="12"/>
                        </svg>
                    </span>
                }.into_any()
            } else { ().into_any() }}
        </button>
    }
}
