use leptos::prelude::*;
use super::types::ToastItem;
use super::toast::Toast;

fn position_class(position: &str) -> &'static str {
    match position {
        "top-left" => "top-4 left-4",
        "top-center" => "top-4 left-1/2 -translate-x-1/2",
        "top-right" => "top-4 right-4",
        "bottom-left" => "bottom-4 left-4",
        "bottom-center" => "bottom-4 left-1/2 -translate-x-1/2",
        _ => "bottom-4 right-4",
    }
}

#[component]
pub fn ToastStack(
    items: RwSignal<Vec<ToastItem>>,
    dismiss: Callback<usize, ()>,
    position: &'static str,
) -> impl IntoView {
    let pos = position_class(position);

    view! {
        <div data-slot="toast-stack"
            class=format!("fixed z-[100] flex flex-col gap-2 max-w-sm w-full p-4 pointer-events-none {}", pos)
        >
            {move || {
                items.get().into_iter().rev().map(|item| {
                    view! {
                        <div class="pointer-events-auto">
                            <Toast item=item dismiss=dismiss />
                        </div>
                    }
                }).collect_view()
            }}
        </div>
    }
}
