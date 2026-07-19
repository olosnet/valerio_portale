use leptos::prelude::*;
use super::command::use_command;

#[component]
pub fn CommandInput(
    #[prop(default = "Cerca...")] placeholder: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_command();
    let extra = class.unwrap_or("");

    let on_input = move |ev: leptos::ev::Event| {
        ctx.query.set(event_target_value(&ev));
    };

    view! {
        <div class="flex items-center border-b px-3" cmdk-input-wrapper="">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-2 size-4 shrink-0 opacity-50">
                <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
            </svg>
            <input
                data-slot="command-input"
                type="text"
                placeholder=placeholder
                on:input=on_input
                class=format!("flex h-11 w-full rounded-md bg-transparent py-3 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50 {}", extra)
            />
        </div>
    }
}
