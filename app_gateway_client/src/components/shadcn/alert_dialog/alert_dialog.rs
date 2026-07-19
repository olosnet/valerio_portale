#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;
use crate::components::shadcn::shared::OverlayProvider;

#[derive(Clone)]
pub struct AlertDialogContext {
    pub open: RwSignal<bool>,
    pub on_action: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[component]
pub fn AlertDialog(
    children: ChildrenFn,
    open: RwSignal<bool>,
    #[prop(optional)] on_action: Option<Arc<dyn Fn() + Send + Sync>>,
) -> impl IntoView {
    provide_context(AlertDialogContext {
        open,
        on_action,
    });

    view! {
        <OverlayProvider open=open>
            {children()}
        </OverlayProvider>
    }
}

pub fn use_alert_dialog() -> AlertDialogContext {
    expect_context::<AlertDialogContext>()
}
