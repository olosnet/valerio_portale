#![allow(dead_code)]
use leptos::prelude::*;
use super::alert_dialog::use_alert_dialog;
use crate::components::shadcn::shared::use_overlay;

#[component]
pub fn AlertDialogAction(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let dialog_ctx = use_alert_dialog();
    let overlay_ctx = use_overlay();
    let extra = class.unwrap_or("");
    let cls = move || format!("inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-destructive text-destructive-foreground shadow hover:bg-destructive/90 h-9 px-4 py-2 {}", extra);

    let handle_click = move |_| {
        if let Some(ref cb) = dialog_ctx.on_action {
            cb();
        }
        overlay_ctx.open.set(false);
    };

    view! {
        <button type="button" on:click=handle_click class=cls()>
            {children()}
        </button>
    }
}
