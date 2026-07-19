use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use leptos::prelude::*;
use super::types::*;

fn schedule_dismiss(id: usize, delay_ms: u32, dismiss_cb: Callback<usize, ()>, dismissed: Rc<Cell<bool>>) {
    let window = web_sys::window().unwrap();
    let cb = Closure::once(move || {
        if !dismissed.get() {
            dismissed.set(true);
            dismiss_cb.run(id);
        }
    });
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            delay_ms as i32,
        )
        .unwrap();
    cb.forget();
}

fn icon_text(toast_type: ToastType) -> &'static str {
    match toast_type {
        ToastType::Success => "\u{2713}",   // ✓
        ToastType::Error => "\u{2715}",     // ✕
        ToastType::Warning => "\u{26A0}",   // ⚠
        ToastType::Info => "\u{2139}",      // ℹ
        ToastType::Default => "",
    }
}

fn border_cls(toast_type: ToastType) -> &'static str {
    match toast_type {
        ToastType::Success => "border-l-green-500",
        ToastType::Error => "border-l-red-500",
        ToastType::Warning => "border-l-yellow-500",
        ToastType::Info => "border-l-blue-500",
        ToastType::Default => "border-l-border",
    }
}

fn color_cls(toast_type: ToastType) -> &'static str {
    match toast_type {
        ToastType::Success => "text-green-500",
        ToastType::Error => "text-red-500",
        ToastType::Warning => "text-yellow-500",
        ToastType::Info => "text-blue-500",
        ToastType::Default => "text-muted-foreground",
    }
}

#[component]
pub fn Toast(
    item: ToastItem,
    dismiss: Callback<usize, ()>,
) -> impl IntoView {
    let visible: RwSignal<bool> = RwSignal::new(true);
    let dismissed_flag: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    let item_id = item.id;
    let callbacks = item.callbacks.clone();
    let on_close = callbacks.as_ref().and_then(|c| c.on_close.clone());

    // Schedule auto-dismiss
    if item.duration_ms > 0 {
        let cb = dismiss;
        let flag = dismissed_flag.clone();
        schedule_dismiss(item_id, item.duration_ms, cb, flag);
    }

    let handle_dismiss = {
        let dismiss = dismiss;
        let flag = dismissed_flag.clone();
        let on_close = on_close.clone();
        move |_| {
            if flag.replace(true) { return; }
            visible.set(false);
            let dim = dismiss;
            let oc = on_close.clone();
            let iid = item_id;
            let window = web_sys::window().unwrap();
            let cb = Closure::once(move || {
                if let Some(ref cb) = oc { cb(); }
                dim.run(iid);
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    200,
                )
                .unwrap();
            cb.forget();
        }
    };

    let type_icon = icon_text(item.toast_type);
    let border = border_cls(item.toast_type);
    let color = color_cls(item.toast_type);

    view! {
        <div data-slot="toast" role="alert"
            class=move || format!(
                "flex items-start gap-3 rounded-lg border bg-background p-4 shadow-lg animate-slide-in-bottom border-l-4 {} {}",
                border,
                if visible.get() { "" } else { "animate-fade-out opacity-0 transition-opacity duration-200" },
            )
        >
            {if !type_icon.is_empty() {
                view! { <span class=format!("mt-0.5 text-sm font-bold shrink-0 {}", color)>{type_icon}</span> }.into_any()
            } else { ().into_any() }}

            <div class="flex-1 min-w-0">
                <p class="text-sm font-medium">{item.title}</p>
                {move || item.description.clone().map(|desc| {
                    view! { <p class="text-sm text-muted-foreground mt-1">{desc}</p> }
                })}
            </div>

            <div class="flex items-center gap-2 shrink-0">
                {move || item.action.clone().map(|act| {
                    let lbl = act.label;
                    let click = act.on_click;
                    let dim = dismiss;
                    let iid = item.id;
                    view! {
                        <button type="button"
                            on:click=move |_| { click(); dim.run(iid); }
                            class="text-sm font-medium text-foreground hover:underline whitespace-nowrap"
                        >
                            {lbl}
                        </button>
                    }
                })}
                <button type="button" on:click=handle_dismiss
                    class="text-muted-foreground hover:text-foreground shrink-0 size-4 inline-flex items-center justify-center"
                    aria-label="Chiudi"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M18 6 6 18"/><path d="m6 6 12 12"/>
                    </svg>
                </button>
            </div>
        </div>
    }
}
