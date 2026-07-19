use std::cell::Cell;
use wasm_bindgen::prelude::*;
use leptos::prelude::*;
use super::panel_group::use_resizable;

/// Raccoglie lo snapshot necessario al drag prima che i Closure
/// catturino i valori.
struct DragState {
    handle_idx: usize,
    left_panel_idx: usize,
    right_panel_idx: usize,
    is_horizontal: bool,
    start_pos: f64,
    left_size: f64,
    right_size: f64,
    min_size: f64,
}

#[component]
pub fn ResizableHandle(
    #[prop(default = false)] with_handle: bool,
) -> impl IntoView {
    let ctx = use_resizable();

    let handle_idx = ctx.handle_counter.get();
    ctx.handle_counter.set(handle_idx + 1);

    // Handle at index H controls panels H and H+1
    let left_idx = handle_idx;
    let right_idx = handle_idx + 1;
    let is_horizontal = ctx.direction == "horizontal";
    let sizes = ctx.sizes;

    let cursor_cls = if is_horizontal { "cursor-col-resize" } else { "cursor-row-resize" };

    let dragging = std::rc::Rc::new(Cell::new(false));

    let handle_mousedown = {
        let dragging = dragging.clone();
        let sizes = sizes;
        move |ev: leptos::ev::MouseEvent| {
            ev.prevent_default();
            let current_sizes = sizes.get();

            if left_idx >= current_sizes.len() || right_idx >= current_sizes.len() {
                return;
            }

            dragging.set(true);
            let dragging_clone = dragging.clone();

            let start_pos = if is_horizontal { ev.client_x() as f64 } else { ev.client_y() as f64 };
            let left_size = current_sizes[left_idx];
            let right_size = current_sizes[right_idx];
            let min_size = 10.0;

            let state = DragState {
                handle_idx,
                left_panel_idx: left_idx,
                right_panel_idx: right_idx,
                is_horizontal,
                start_pos,
                left_size,
                right_size,
                min_size,
            };

            let sizes_clone = sizes;
            let dragging_clone = dragging.clone();

            // mousemove
            let move_cb: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MouseEvent)> = Closure::new(move |ev: web_sys::MouseEvent| {
                let current_pos = if state.is_horizontal { ev.client_x() as f64 } else { ev.client_y() as f64 };
                let delta = current_pos - state.start_pos;
                let total = state.left_size + state.right_size;
                let new_left = (state.left_size + delta).clamp(state.min_size, total - state.min_size);
                let new_right = total - new_left;

                sizes_clone.update(|s| {
                    if state.left_panel_idx < s.len() && state.right_panel_idx < s.len() {
                        s[state.left_panel_idx] = new_left;
                        s[state.right_panel_idx] = new_right;
                    }
                });
            });

            // mouseup
            let up_cb: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MouseEvent)> = Closure::new(move |_ev: web_sys::MouseEvent| {
                dragging_clone.set(false);
            });

            let window = web_sys::window().unwrap();

            window.add_event_listener_with_callback("mousemove", move_cb.as_ref().unchecked_ref()).unwrap();
            window.add_event_listener_with_callback("mouseup", up_cb.as_ref().unchecked_ref()).unwrap();

            move_cb.forget();
            up_cb.forget();
        }
    };

    let handle_cls = move || {
        let base = format!(
            "relative flex items-center justify-center after:absolute after:inset-y-0 after:left-1/2 after:w-1 after:-translate-x-1/2 after:rounded-full after:bg-border hover:after:bg-muted-foreground/50 transition-all {}",
            cursor_cls,
        );
        if with_handle {
            format!("{} min-w-4 min-h-4", base)
        } else {
            format!("{} min-w-[5px] min-h-[5px]", base)
        }
    };

    view! {
        <div data-slot="resizable-handle"
            role="separator"
            tabindex="0"
            on:mousedown=handle_mousedown
            class=handle_cls()
        >
            {if with_handle {
                view! {
                    <div class="z-10 flex items-center justify-center rounded-full border bg-background shadow-sm size-3">
                        <svg xmlns="http://www.w3.org/2000/svg" width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                            class=if is_horizontal { "rotate-90" } else { "" }
                        >
                            <circle cx="9" cy="12" r="1"/><circle cx="15" cy="12" r="1"/>
                        </svg>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </div>
    }
}
