#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn TableFooter(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("border-t bg-muted/50 font-medium [&>tr]:last:border-b-0 {}", extra);

    view! {
        <tfoot data-slot="table-footer" class=cls()>
            {children()}
        </tfoot>
    }
}
