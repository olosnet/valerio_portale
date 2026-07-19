use std::sync::Arc;
use leptos::prelude::*;
use valerios_ui_toolkit::sonner::{ToastContext, ToastInput, ToastType};

/// Recupera il contesto toast per mostrare notifiche.
pub fn use_toast_ctx() -> ToastContext {
    use_context::<ToastContext>().expect("Sonner not mounted — add <Sonner> to app root")
}

/// Mostra un toast di errore.
pub fn toast_error(ctx: &ToastContext, msg: &str) {
    ctx.add.run(ToastInput {
        title: msg.to_string(),
        description: None,
        toast_type: ToastType::Error,
        action: None,
        duration_ms: 5000,
        callbacks: None,
    });
}

/// Mostra un toast di successo.
pub fn toast_success(ctx: &ToastContext, msg: &str) {
    ctx.add.run(ToastInput {
        title: msg.to_string(),
        description: None,
        toast_type: ToastType::Success,
        action: None,
        duration_ms: 3000,
        callbacks: None,
    });
}

/// Mostra un toast informativo.
pub fn toast_info(ctx: &ToastContext, msg: &str) {
    ctx.add.run(ToastInput {
        title: msg.to_string(),
        description: None,
        toast_type: ToastType::Info,
        action: None,
        duration_ms: 3000,
        callbacks: None,
    });
}

/// Mostra un toast di errore con azione (es. "Annulla").
pub fn toast_error_with_action(ctx: &ToastContext, msg: &str, action_label: &'static str, on_action: Arc<dyn Fn() + Send + Sync>) {
    ctx.add.run(ToastInput {
        title: msg.to_string(),
        description: None,
        toast_type: ToastType::Error,
        action: Some(valerios_ui_toolkit::sonner::ToastAction {
            label: action_label,
            on_click: on_action,
        }),
        duration_ms: 5000,
        callbacks: None,
    });
}
