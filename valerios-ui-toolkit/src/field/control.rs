use leptos::prelude::*;
use super::field::use_field;

#[component]
pub fn FieldControl(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_field();
    let extra = class.unwrap_or("");

    view! {
        <div data-slot="field-control" id=format!("{}-control", ctx.id) class=extra>
            {children()}
        </div>
    }
}
