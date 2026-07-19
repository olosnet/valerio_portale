#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn TableCell(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("p-4 align-middle [&:has([role=checkbox])]:pr-0 {}", extra);

    view! {
        <td data-slot="table-cell" class=cls()>
            {children()}
        </td>
    }
}
