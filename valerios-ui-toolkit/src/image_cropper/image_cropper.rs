use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::button::{Button, ButtonVariant};
use crate::dialog::*;

async fn load_img_dimensions(url: &str) -> (u32, u32) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return (256, 256),
    };
    let resp = match JsFuture::from(window.fetch_with_str(url)).await {
        Ok(r) => r,
        _ => return (256, 256),
    };
    let resp: web_sys::Response = match resp.dyn_into() {
        Ok(r) => r,
        _ => return (256, 256),
    };
    let blob = match resp.blob() { Ok(b) => b, _ => return (256, 256) };
    let blob = match JsFuture::from(blob).await {
        Ok(b) => b,
        _ => return (256, 256),
    };
    let blob: web_sys::Blob = match blob.dyn_into() {
        Ok(b) => b,
        _ => return (256, 256),
    };
    let bitmap_promise = match window.create_image_bitmap_with_blob(&blob) {
        Ok(p) => p,
        _ => return (256, 256),
    };
    let bitmap = match JsFuture::from(bitmap_promise).await {
        Ok(b) => b,
        _ => return (256, 256),
    };
    let bitmap: web_sys::ImageBitmap = match bitmap.dyn_into() {
        Ok(b) => b,
        _ => return (256, 256),
    };
    (bitmap.width(), bitmap.height())
}

pub async fn crop_via_canvas(
    image_url: &str,
    sx: f64,
    sy: f64,
    crop_sz: f64,
    out_sz: Option<u32>,
) -> Result<(Vec<u8>, String), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;

    let resp = JsFuture::from(window.fetch_with_str(image_url)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let blob = JsFuture::from(resp.blob()?).await?.dyn_into::<web_sys::Blob>()?;
    let bitmap_promise = window.create_image_bitmap_with_blob(&blob)?;
    let bitmap: web_sys::ImageBitmap = JsFuture::from(bitmap_promise).await?.dyn_into()?;

    let crop_int = crop_sz.ceil() as u32;
    let crop_canvas = web_sys::OffscreenCanvas::new(crop_int, crop_int)?;
    let crop_ctx = crop_canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("No 2d context"))?
        .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()?;

    crop_ctx.draw_image_with_image_bitmap_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        &bitmap, sx, sy, crop_sz, crop_sz, 0.0, 0.0, crop_int as f64, crop_int as f64,
    )?;

    let final_canvas = match out_sz {
        Some(n) if n != crop_int => {
            let resize_canvas = web_sys::OffscreenCanvas::new(n, n)?;
            let resize_ctx = resize_canvas
                .get_context("2d")?
                .ok_or_else(|| JsValue::from_str("No 2d context"))?
                .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()?;
            resize_ctx.draw_image_with_offscreen_canvas_and_dw_and_dh(
                &crop_canvas, 0.0, 0.0, n as f64, n as f64,
            )?;
            resize_canvas
        }
        _ => crop_canvas,
    };

    let blob_p = final_canvas.convert_to_blob()?;
    let blob = JsFuture::from(blob_p).await?;
    let ab_fn = js_sys::Reflect::get(&blob, &"arrayBuffer".into())
        .and_then(|v| v.dyn_into::<js_sys::Function>())?;
    let ab_p = ab_fn.call0(&blob)
        .and_then(|v| v.dyn_into::<js_sys::Promise>())?;
    let array_buffer = JsFuture::from(ab_p).await?;
    let uint8 = js_sys::Uint8Array::new(&array_buffer);
    let mut bytes = vec![0u8; uint8.length() as usize];
    uint8.copy_to(&mut bytes);
    Ok((bytes, "png".to_string()))
}

#[component]
pub fn ImageCropper(
    open: RwSignal<bool>,
    image_bytes: Vec<u8>,
    image_url: String,
    on_crop: Callback<(Vec<u8>, String)>,
) -> impl IntoView {
    let display_sz = 256.0;
    let img_src = RwSignal::new(image_url.clone());
    let dims = RwSignal::new((256u32, 256u32));
    let nw = Signal::derive(move || dims.get().0);
    let nh = Signal::derive(move || dims.get().1);

    let dims_url = img_src.get();
    spawn_local(async move {
        let d = load_img_dimensions(&dims_url).await;
        if d.0 > 1 && d.1 > 1 { dims.set(d); }
    });

    let zoom = RwSignal::new(1.0);
    let ox = RwSignal::new(0.0);
    let oy = RwSignal::new(0.0);

    Effect::new(move |_| {
        let (w, h) = dims.get();
        if w <= 1 && h <= 1 { return; }
        let z = (display_sz / w as f64).max(display_sz / h as f64);
        zoom.set(z);
        ox.set((display_sz - w as f64 * z) / 2.0);
        oy.set((display_sz - h as f64 * z) / 2.0);
    });

    let (dragging, set_dragging) = signal(false);
    let (drag_start_x, set_drag_start_x) = signal(0.0);
    let (drag_start_y, set_drag_start_y) = signal(0.0);
    let (drag_orig_ox, set_drag_orig_ox) = signal(0.0);
    let (drag_orig_oy, set_drag_orig_oy) = signal(0.0);

    let interacted = RwSignal::new(false);
    let processing = RwSignal::new(false);

    let ptr_down = Callback::new(move |ev: leptos::ev::PointerEvent| {
        set_drag_start_x.set(ev.client_x() as f64);
        set_drag_start_y.set(ev.client_y() as f64);
        set_drag_orig_ox.set(ox.get());
        set_drag_orig_oy.set(oy.get());
        set_dragging.set(true);
        interacted.set(true);
    });

    let ptr_move = Callback::new(move |ev: leptos::ev::PointerEvent| {
        if dragging.get() {
            ox.set(drag_orig_ox.get() + ev.client_x() as f64 - drag_start_x.get());
            oy.set(drag_orig_oy.get() + ev.client_y() as f64 - drag_start_y.get());
        }
    });

    let ptr_up = Callback::new(move |_: leptos::ev::PointerEvent| {
        set_dragging.set(false);
    });

    let on_zoom = Callback::new(move |ev: leptos::ev::Event| {
        let z: f64 = event_target_value(&ev).parse().unwrap_or(1.0);
        zoom.set(z);
    });

    let confirm = Callback::new(move |_: ()| {
        if processing.get() { return; }
        processing.set(true);

        let z = zoom.get_untracked();
        let ox_val = -ox.get_untracked();
        let oy_val = -oy.get_untracked();
        let (w, h) = dims.get_untracked();
        let crop_sz_px = (display_sz / z) as u32;
        let size = crop_sz_px.min(w).min(h) as f64;
        let left = (ox_val / z).max(0.0);
        let top = (oy_val / z).max(0.0);
        let url = img_src.get();

        spawn_local(async move {
            match crop_via_canvas(&url, left, top, size, None).await
            {
                Ok(result) => {
                    processing.set(false);
                    on_crop.run(result);
                }
                Err(e) => {
                    processing.set(false);
                    leptos::logging::warn!("Crop error: {:?}", e);
                }
            }
        });
    });

    view! {
        <Dialog open=open>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>"Ritaglia immagine profilo"</DialogTitle>
                    <DialogDescription>
                        "Trascina l'immagine per posizionarla. Usa lo zoom per ingrandire."
                    </DialogDescription>
                </DialogHeader>

                <div class="flex flex-col items-center gap-3">
                    <div
                        class="relative rounded-full overflow-hidden border-2 border-border bg-muted shrink-0 select-none touch-none"
                        style=move || format!("width:{}px;height:{}px;cursor:{}", display_sz, display_sz,
                            if dragging.get() { "grabbing" } else { "grab" })
                        on:pointerdown=move |ev| ptr_down.run(ev)
                        on:pointermove=move |ev| ptr_move.run(ev)
                        on:pointerup=move |ev| ptr_up.run(ev)
                    >
                        <img src=img_src
                            class="absolute max-w-none pointer-events-none"
                            style:width=move || format!("{}px", nw.get())
                            style:height=move || format!("{}px", nh.get())
                            style:transform=move || format!(
                                "translate({:.1}px,{:.1}px) scale({:.4})",
                                ox.get(), oy.get(), zoom.get()
                            )
                            style="transform-origin:0 0"
                            draggable="false"
                        />
                        <div class="absolute inset-0 bg-black/25 pointer-events-none" style="z-index:5" />
                        <div class="absolute inset-0 rounded-full pointer-events-none" style="z-index:10">
                            <div class="absolute inset-0 rounded-full border-[3px] border-dashed border-white/70 dark:border-white/50" />
                            <div class="absolute top-[33%] left-0 right-0 h-px bg-white/60" />
                            <div class="absolute top-[66%] left-0 right-0 h-px bg-white/60" />
                            <div class="absolute top-0 bottom-0 left-[33%] w-px bg-white/60" />
                            <div class="absolute top-0 bottom-0 left-[66%] w-px bg-white/60" />
                            <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-24 h-24 rounded-full border border-white/30" />
                            <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-1.5 h-1.5 rounded-full bg-white/60" />
                        </div>
                        {move || if !interacted.get() && !processing.get() {
                            view! {
                                <div class="absolute inset-0 flex items-center justify-center pointer-events-none" style="z-index:15">
                                    <span class="text-xs text-white font-medium bg-black/50 px-3 py-1.5 rounded-full">
                                        "Trascina per posizionare"
                                    </span>
                                </div>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>

                    <div class="w-full max-w-[256px] flex items-center gap-2">
                        <span class="text-xs text-muted-foreground shrink-0">"Zoom"</span>
                        <input type="range" min="0.01" max="3.0" step="0.01"
                            prop:value=move || zoom.get().to_string()
                            on:input=move |ev| on_zoom.run(ev)
                            class="w-full h-2 bg-input rounded-lg appearance-none cursor-pointer accent-primary"
                        />
                    </div>
                </div>

                <DialogFooter>
                    <Button variant=ButtonVariant::Default
                        on_click=Arc::new(move || confirm.run(()))>"Conferma"</Button>
                    <DialogClose>
                        <Button variant=ButtonVariant::Outline>"Annulla"</Button>
                    </DialogClose>
                </DialogFooter>
            </DialogContent>

            {move || if processing.get() {
                view! {
                    <div class="fixed inset-0 z-[60] flex flex-col items-center justify-center bg-background/90">
                        <div class="size-10 rounded-full border-[3px] border-primary/25 border-t-primary animate-spin" />
                        <span class="text-sm font-medium text-foreground mt-3">"Attendere..."</span>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </Dialog>
    }
}
