use leptos::prelude::*;

#[component]
pub fn Textarea(
    value: RwSignal<String>,
    #[prop(default = "")] placeholder: &'static str,
    #[prop(default = false)] disabled: bool,
    #[prop(default = 3)] rows: u16,
    #[prop(optional)] id: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let base = "flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-base ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm";
    let extra = class.unwrap_or("");
    let cls = move || format!("{} {}", base, extra);

    let on_input = move |ev: leptos::ev::Event| {
        value.set(event_target_value(&ev));
    };

    view! {
        <textarea
            id=id
            placeholder=placeholder
            disabled=disabled
            rows=rows.to_string()
            on:input=on_input
            class=cls()
        >{move || value.get()}</textarea>
    }
}
