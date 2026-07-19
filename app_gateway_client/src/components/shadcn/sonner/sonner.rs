#![allow(dead_code)]
use leptos::prelude::*;
use super::types::*;
use super::toast_stack::ToastStack;

#[derive(Clone, Copy)]
pub struct ToastContext {
    pub add: Callback<ToastInput, usize>,
    pub dismiss: Callback<usize, ()>,
}

pub fn use_toast() -> ToastContext {
    expect_context::<ToastContext>()
}

#[component]
pub fn Sonner(
    #[prop(default = "bottom-right")] position: &'static str,
    #[prop(default = 4000)] default_duration_ms: u32,
    #[prop(default = 5)] max_visible: usize,
    children: Children,
) -> impl IntoView {
    let items: RwSignal<Vec<ToastItem>> = RwSignal::new(Vec::new());
    let next_id: RwSignal<usize> = RwSignal::new(1);

    let context = {
        let items = items;
        let next_id = next_id;
        let default_duration_ms = default_duration_ms;
        let max_visible = max_visible;

        let dismiss_clone = items.clone();
        let dismiss = Callback::new(move |id: usize| {
            dismiss_clone.update(|list| list.retain(|t| t.id != id));
        });

        let add = Callback::new(move |input: ToastInput| {
            let id = next_id.get();
            next_id.set(id + 1);
            let duration = if input.duration_ms > 0 { input.duration_ms } else { default_duration_ms };

            let item = ToastItem {
                id,
                title: input.title,
                description: input.description,
                toast_type: input.toast_type,
                action: input.action,
                duration_ms: duration,
                callbacks: input.callbacks,
            };

            items.update(|list| {
                list.push(item.clone());
                if list.len() > max_visible {
                    list.remove(0);
                }
            });

            id
        });

        ToastContext { add, dismiss }
    };

    provide_context(context.clone());

    view! {
        <div data-slot="sonner" class="relative">
            {children()}
            <ToastStack items=items dismiss=context.dismiss position=position />
        </div>
    }
}
