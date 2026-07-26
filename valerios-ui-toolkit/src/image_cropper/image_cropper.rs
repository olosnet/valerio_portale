use std::io::Cursor;
use std::sync::Arc;

use image::ImageReader;
use leptos::prelude::*;

use crate::button::{Button, ButtonVariant};
use crate::dialog::*;

pub fn crop_and_resize(
    bytes: &[u8], x: u32, y: u32, crop_sz: u32, out_sz: u32,
) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let cropped = img.crop_imm(x, y, crop_sz, crop_sz);
    let resized = cropped.resize_exact(out_sz, out_sz, image::imageops::FilterType::Lanczos3);
    let mut buf = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(buf)
}

fn compute_dimensions(bytes: &[u8]) -> (u32, u32) {
    ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.into_dimensions().ok())
        .unwrap_or((1, 1))
}

fn compute_crop_params(
    zoom: f64, ox: f64, oy: f64, display_sz: f64, nw: u32, nh: u32,
) -> (u32, u32, u32) {
    let left = (ox / zoom).max(0.0) as u32;
    let top = (oy / zoom).max(0.0) as u32;
    let size = ((display_sz / zoom) as u32).min(nw).min(nh);
    (left, top, size)
}

#[component]
pub fn ImageCropper(
    open: RwSignal<bool>,
    image_bytes: Vec<u8>,
    image_url: String,
    #[prop(default = 256)] output_size: u32,
    on_crop: Callback<Vec<u8>>,
) -> impl IntoView {
    let (nw, nh) = compute_dimensions(&image_bytes);
    let display_sz = 256.0;
    let image_url = StoredValue::new(image_url);
    let image_bytes = StoredValue::new(image_bytes);

    let initial_z = (display_sz / nw as f64).max(display_sz / nh as f64);
    let zoom = RwSignal::new(initial_z);
    let ox = RwSignal::new((display_sz - nw as f64 * initial_z) / 2.0);
    let oy = RwSignal::new((display_sz - nh as f64 * initial_z) / 2.0);

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
        let z: f64 = event_target_value(&ev).parse().unwrap_or(initial_z);
        zoom.set(z.clamp(initial_z, initial_z * 3.0));
    });

    let confirm = Callback::new(move |_: ()| {
        if processing.get() { return; }
        processing.set(true);

        let z = zoom.get_untracked();
        let x = -ox.get_untracked();
        let y = -oy.get_untracked();
        let (left, top, size) = compute_crop_params(z, x, y, display_sz, nw, nh);
        let bytes = image_bytes.read_value().clone();
        let oc = on_crop;

        leptos::task::spawn_local(async move {
            match crop_and_resize(&bytes, left, top, size, output_size) {
                Ok(r) => {
                    processing.set(false);
                    oc.run(r);
                }
                Err(e) => {
                    processing.set(false);
                    leptos::logging::warn!("Crop error: {e}");
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
                        <img src=image_url.read_value().clone()
                            class="absolute max-w-none pointer-events-none"
                            style:width=format!("{}px", nw)
                            style:height=format!("{}px", nh)
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
                        <input type="range" min="0.01" max=move || (initial_z * 3.0).to_string() step="0.01"
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
                        <svg class="animate-spin size-10 text-primary" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
                        </svg>
                        <span class="text-sm font-medium text-foreground mt-3">"Attendere..."</span>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </Dialog>
    }
}
