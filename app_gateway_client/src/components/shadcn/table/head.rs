#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn TableHead(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("h-12 px-4 text-left align-middle font-medium text-muted-foreground [&:has([role=checkbox])]:pr-0 {}", extra);

    view! {
        <th data-slot="table-head" class=cls()>
            {children()}
        </th>
    }
}
