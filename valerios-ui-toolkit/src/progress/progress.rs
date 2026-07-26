use leptos::prelude::*;

#[component]
pub fn Progress(
    value: RwSignal<u8>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || {
        format!(
            "relative h-4 w-full overflow-hidden rounded-full bg-secondary {}",
            extra
        )
    };
    // let pct = move || {
    //     let v = value.get().min(100);
    //     format!("{}%", v)
    // };
    //
    view! {
        <div data-slot="progress" role="progressbar" aria-valuenow=move || value.get().to_string() aria-valuemin="0" aria-valuemax="100" class=cls()>
            <div
                data-slot="progress-indicator"
                class="h-full w-full flex-1 bg-primary transition-all"
                style=move || format!("transform: translateX(-{}%)", 100 - value.get().min(100))
            />
        </div>
    }
}
