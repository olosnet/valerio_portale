use leptos::prelude::*;
use super::select::use_select;

#[component]
pub fn SelectValue(
    #[prop(default = "Seleziona...")] placeholder: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_select();
    let extra = class.unwrap_or("");

    let text = move || {
        let v = ctx.value.get();
        if v.is_empty() { placeholder.to_string() } else { v }
    };

    view! {
        <span data-slot="select-value" class=format!("truncate {}", extra)>
            {text()}
        </span>
    }
}
