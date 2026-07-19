use leptos::prelude::*;

#[component]
pub fn Empty(
    #[prop(default = "Nessun dato")] title: &'static str,
    #[prop(default = "")] description: &'static str,
    #[prop(optional)] icon: Option<leptos::prelude::AnyView>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    let icon_render = icon;
    view! {
        <div data-slot="empty" class=format!("flex flex-col items-center justify-center py-12 text-center {}", extra)>
            {icon_render}
            <h3 class="mt-4 text-lg font-semibold text-foreground">{title}</h3>
            {if !description.is_empty() {
                view! { <p class="mt-2 text-sm text-muted-foreground">{description}</p> }.into_any()
            } else { ().into_any() }}
        </div>
    }
}
