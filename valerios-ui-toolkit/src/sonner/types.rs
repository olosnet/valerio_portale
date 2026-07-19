use std::sync::Arc;

#[derive(Clone, Copy, PartialEq)]
pub enum ToastType {
    Default,
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone)]
pub struct ToastAction {
    pub label: &'static str,
    pub on_click: Arc<dyn Fn() + Send + Sync>,
}

#[derive(Clone)]
pub struct ToastCallbacks {
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Clone)]
pub struct ToastItem {
    pub id: usize,
    pub title: String,
    pub description: Option<String>,
    pub toast_type: ToastType,
    pub action: Option<ToastAction>,
    pub duration_ms: u32,
    pub callbacks: Option<ToastCallbacks>,
}

pub struct ToastInput {
    pub title: String,
    pub description: Option<String>,
    pub toast_type: ToastType,
    pub action: Option<ToastAction>,
    pub duration_ms: u32,
    pub callbacks: Option<ToastCallbacks>,
}
