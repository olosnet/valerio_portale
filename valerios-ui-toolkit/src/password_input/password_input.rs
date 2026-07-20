use leptos::prelude::*;

#[component]
pub fn PasswordInput(
    value: RwSignal<String>,
    #[prop(default = "Password")] placeholder: &'static str,
    #[prop(optional)] disabled: Option<Signal<bool>>,
    #[prop(optional)] id: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let show_password = RwSignal::new(false);
    let input_type = move || {
        if show_password.get() { "text" } else { "password" }
    };

    let base = "flex h-10 w-full rounded-md border border-input bg-background text-foreground px-3 py-2 text-base ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm pr-10";
    let extra = class.unwrap_or("");
    let cls = move || format!("{} {}", base, extra);

    let on_input = move |ev: leptos::ev::Event| {
        value.set(event_target_value(&ev));
    };

    let toggle_show = move |_| show_password.update(|v| *v = !*v);

    let is_disabled = move || disabled.map(|d| d.get()).unwrap_or(false);

    view! {
        <div class="relative">
            <input
                id=id
                type=input_type
                placeholder=placeholder
                disabled=is_disabled
                on:input=on_input
                prop:value=value
                class=cls()
            />
            <button
                type="button"
                on:click=toggle_show
                tabindex="-1"
                disabled=is_disabled
                class="absolute inset-y-0 right-0 flex items-center pr-3 text-muted-foreground hover:text-foreground disabled:opacity-50"
                aria-label=move || {
                    if show_password.get() { "Nascondi password" } else { "Mostra password" }
                }
            >
                {move || if show_password.get() {
                    view! {
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4 shrink-0">
                            <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
                            <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24"/>
                            <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/>
                            <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/>
                            <line x1="2" x2="22" y1="2" y2="22"/>
                        </svg>
                    }.into_any()
                } else {
                    view! {
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4 shrink-0">
                            <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
                            <circle cx="12" cy="12" r="3"/>
                        </svg>
                    }.into_any()
                }}
            </button>
        </div>
    }
}
