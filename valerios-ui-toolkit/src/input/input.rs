use leptos::prelude::*;

#[component]
pub fn Input(
    #[prop(default = "text")] input_type: &'static str,
    #[prop(default = "")] placeholder: &'static str,
    value: RwSignal<String>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] id: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let base = "flex h-10 w-full rounded-md border border-input bg-background text-foreground px-3 py-2 text-base ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm";
    let extra = class.unwrap_or("");
    let cls = move || format!("{} {}", base, extra);

    let on_input = move |ev: leptos::ev::Event| {
        value.set(event_target_value(&ev));
    };

    view! {
        <input
            id=id
            type=input_type
            placeholder=placeholder
            disabled=disabled
            on:input=on_input
            class=cls()
        />
    }
}
