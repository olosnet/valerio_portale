#![allow(dead_code)]
use leptos::prelude::*;
use crate::components::shadcn::icon::Icon;

fn icon_chevron() -> AnyView { Icon::ChevronDown.render() }

#[component]
pub fn Combobox(
    _value: RwSignal<String>,
    _options: Vec<(&'static str, &'static str)>,
    #[prop(default = "Seleziona...")] _placeholder: &'static str,
    #[prop(default = "Cerca...")] _search_placeholder: &'static str,
    #[prop(default = "Nessun risultato.")] _empty_text: &'static str,
    #[prop(optional)] _on_change: Option<std::sync::Arc<dyn Fn(String) + Send + Sync>>,
) -> impl IntoView {
    view! {
        <p>"Combobox (non implementato)"</p>
    }
}
