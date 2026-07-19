#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn Kbd(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <kbd data-slot="kbd" class=format!("pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100 {}", extra)>
            {children()}
        </kbd>
    }
}
