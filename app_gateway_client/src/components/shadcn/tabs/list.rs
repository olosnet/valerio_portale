#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn TabsList(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("inline-flex h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground {}", extra);

    view! {
        <div role="tablist" data-slot="tabs-list" class=cls()>
            {children()}
        </div>
    }
}
