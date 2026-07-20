use std::io::Cursor;
use std::sync::Arc;

use image::GenericImageView;
use image::ImageReader;
use leptos::prelude::*;

use crate::button::{Button, ButtonVariant};
use crate::dialog::*;

pub fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() >= 2 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() == 3 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn to_data_url(bytes: &[u8], mime: &str) -> String {
    format!("data:{mime};base64,{}", base64_encode(bytes))
}

pub fn crop_and_resize(
    bytes: &[u8],
    x: u32,
    y: u32,
    crop_sz: u32,
    out_sz: u32,
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

#[component]
pub fn ImageCropper(
    open: RwSignal<bool>,
    image_bytes: Vec<u8>,
    #[prop(default = 256)] output_size: u32,
    on_crop: Callback<Vec<u8>>,
) -> impl IntoView {
    // Solo header, NON decodifica l'intera immagine (istantaneo vs secondi)
    let (natural_w, natural_h) = ImageReader::new(Cursor::new(&image_bytes))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.into_dimensions().ok())
        .unwrap_or((1, 1));

    let display_sz: f64 = 256.0;
    let initial_zoom = (display_sz / natural_w as f64).max(display_sz / natural_h as f64);
    let zoom = RwSignal::new(initial_zoom);
    let offset_x = RwSignal::new((display_sz - natural_w as f64 * initial_zoom) / 2.0);
    let offset_y = RwSignal::new((display_sz - natural_h as f64 * initial_zoom) / 2.0);

    let min_zoom = initial_zoom;
    let max_zoom = initial_zoom * 3.0;

    // Rileva MIME dal magic number (evita re-encoding)
    let mime = if image_bytes.starts_with(&[0xFF, 0xD8]) {
        "image/jpeg"
    } else if image_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else {
        "image/png"
    };
    let image_data_url = StoredValue::new(to_data_url(&image_bytes, mime));
    let image_bytes = StoredValue::new(image_bytes);

    let (dragging, set_dragging) = signal(false);
    let (drag_start_x, set_drag_start_x) = signal(0.0);
    let (drag_start_y, set_drag_start_y) = signal(0.0);
    let (drag_orig_x, set_drag_orig_x) = signal(0.0);
    let (drag_orig_y, set_drag_orig_y) = signal(0.0);

    let preview_bytes_rw = RwSignal::new(Vec::<u8>::new());
    let preview_data_url = StoredValue::new(String::new());

    let (interacted, set_interacted) = signal(false);
    let processing = RwSignal::new(false);

    let handle_ptr_down = Callback::new({
        let set_drag_start_x = set_drag_start_x;
        let set_drag_start_y = set_drag_start_y;
        let set_drag_orig_x = set_drag_orig_x;
        let set_drag_orig_y = set_drag_orig_y;
        let set_dragging = set_dragging;
        let set_interacted = set_interacted;
        move |ev: leptos::ev::PointerEvent| {
            set_drag_start_x.set(ev.client_x() as f64);
            set_drag_start_y.set(ev.client_y() as f64);
            set_drag_orig_x.set(offset_x.get());
            set_drag_orig_y.set(offset_y.get());
            set_dragging.set(true);
            set_interacted.set(true);
        }
    });

    let handle_ptr_move = Callback::new(move |ev: leptos::ev::PointerEvent| {
        if dragging.get() {
            let dx = ev.client_x() as f64 - drag_start_x.get();
            let dy = ev.client_y() as f64 - drag_start_y.get();
            offset_x.set(drag_orig_x.get() + dx);
            offset_y.set(drag_orig_y.get() + dy);
        }
    });

    let handle_ptr_up = Callback::new(move |_: leptos::ev::PointerEvent| {
        set_dragging.set(false);
    });

    let on_zoom = Callback::new(move |ev: leptos::ev::Event| {
        let new_z: f64 = event_target_value(&ev).parse().unwrap_or(initial_zoom);
        zoom.set(new_z.clamp(min_zoom, max_zoom));
    });

    let handle_confirm = Callback::new(move |_: ()| {
        if processing.get() { return; }
        processing.set(true);
        let bytes = image_bytes.read_value();
        let z = zoom.get();
        let ox = -offset_x.get();
        let oy = -offset_y.get();
        let crop_left = (ox / z).max(0.0) as u32;
        let crop_top = (oy / z).max(0.0) as u32;
        let crop_size = ((display_sz / z) as u32).min(natural_w).min(natural_h);

        match crop_and_resize(&bytes, crop_left, crop_top, crop_size, output_size) {
            Ok(result) => {
                on_crop.run(result);
            }
            Err(e) => {
                processing.set(false);
                leptos::logging::warn!("Crop failed: {e}");
            }
        }
    });

    let handle_preview = Callback::new(move |_: ()| {
        let bytes = image_bytes.read_value();
        let z = zoom.get();
        let ox = -offset_x.get();
        let oy = -offset_y.get();
        let crop_left = (ox / z).max(0.0) as u32;
        let crop_top = (oy / z).max(0.0) as u32;
        let crop_size = ((display_sz / z) as u32).min(natural_w).min(natural_h);

        if let Ok(result) = crop_and_resize(&bytes, crop_left, crop_top, crop_size, 128) {
            let url = to_data_url(&result, "image/png");
            preview_data_url.set_value(url);
            preview_bytes_rw.set(result);
        }
    });

    view! {
        <Dialog open=open>
            <DialogContent>
                <div class="relative">
                    <DialogHeader>
                        <DialogTitle>"Ritaglia immagine profilo"</DialogTitle>
                        <DialogDescription>
                            "Trascina l'immagine per posizionarla. Usa lo zoom per ingrandire o rimpicciolire."
                        </DialogDescription>
                    </DialogHeader>

                <div class="flex flex-col items-center gap-4">
                    <div class="flex gap-6">
                        <div
                            class="relative rounded-full overflow-hidden border-2 border-border bg-muted shrink-0 select-none touch-none"
                            style=move || format!("width:{}px;height:{}px;cursor:{}", display_sz, display_sz, if dragging.get() { "grabbing" } else { "grab" })
                            on:pointerdown=move |ev| handle_ptr_down.run(ev)
                            on:pointermove=move |ev| handle_ptr_move.run(ev)
                            on:pointerup=move |ev| handle_ptr_up.run(ev)
                        >
                            <img
                                src=image_data_url.read_value().clone()
                                class="absolute max-w-none pointer-events-none"
                                style:width=format!("{}px", natural_w)
                                style:height=format!("{}px", natural_h)
                                style:transform=move || format!(
                                    "translate({:.1}px, {:.1}px) scale({:.4})",
                                    offset_x.get(), offset_y.get(), zoom.get()
                                )
                                style="transform-origin:0 0"
                                draggable="false"
                                alt="Crop preview"
                            />
                            <div class="absolute inset-0 bg-black/25 pointer-events-none" style="z-index:5" />
                            <div class="absolute inset-0 rounded-full border-[3px] border-dashed border-white/80 dark:border-white/60 pointer-events-none" style="z-index:10">
                                <div class="absolute top-1/3 left-0 right-0 h-0.5 bg-white/60 pointer-events-none" />
                                <div class="absolute top-2/3 left-0 right-0 h-0.5 bg-white/60 pointer-events-none" />
                                <div class="absolute left-1/3 top-0 bottom-0 w-0.5 bg-white/60 pointer-events-none" />
                                <div class="absolute left-2/3 top-0 bottom-0 w-0.5 bg-white/60 pointer-events-none" />
                                <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-24 h-24 rounded-full border border-white/40 pointer-events-none" />
                                <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-1.5 h-1.5 rounded-full bg-white/70 pointer-events-none" />
                            </div>
                            {move || {
                                if !interacted.get() && !processing.get() {
                                    view! {
                                        <div class="absolute inset-0 flex items-center justify-center pointer-events-none" style="z-index:15">
                                            <span class="text-xs text-white font-medium bg-black/50 px-3 py-1.5 rounded-full">"Trascina per posizionare"</span>
                                        </div>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }
                            }}
                        </div>

                        <div class="flex flex-col items-center gap-2">
                            <span class="text-xs text-muted-foreground">"Anteprima"</span>
                            {move || {
                                let bytes = preview_bytes_rw.get();
                                if bytes.is_empty() {
                                    view! {
                                        <div class="size-24 rounded-full bg-muted border border-border shrink-0" />
                                    }.into_any()
                                } else {
                                    let url = to_data_url(&bytes, "image/png");
                                    view! {
                                        <img src=url class="size-24 rounded-full object-cover border border-border shrink-0" alt="Preview" />
                                    }.into_any()
                                }
                            }}
                            <Button
                                variant=ButtonVariant::Outline
                                on_click=Arc::new(move || handle_preview.run(()))
                            >"Aggiorna anteprima"</Button>
                        </div>
                    </div>

                    <div class="w-full max-w-[256px] flex items-center gap-2">
                        <span class="text-xs text-muted-foreground shrink-0">"Zoom"</span>
                        <input
                            type="range"
                            min=min_zoom.to_string()
                            max=max_zoom.to_string()
                            step="0.01"
                            prop:value=move || zoom.get().to_string()
                            on:input=move |ev| on_zoom.run(ev)
                            class="w-full h-2 bg-input rounded-lg appearance-none cursor-pointer accent-primary"
                        />
                    </div>
                </div>

                <DialogFooter>
                    <Button variant=ButtonVariant::Default on_click=Arc::new(move || handle_confirm.run(()))>"Conferma"</Button>
                    <DialogClose>
                        <Button variant=ButtonVariant::Outline>"Annulla"</Button>
                    </DialogClose>
                </DialogFooter>

                    {move || if processing.get() {
                        view! {
                            <div class="absolute inset-0 z-50 flex flex-col items-center justify-center rounded-lg bg-background/90 backdrop-blur-sm">
                                <div class="size-10 animate-spin rounded-full border-3 border-muted border-t-primary" />
                                <span class="text-sm font-medium text-foreground mt-3">"Attendere..."</span>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }}
                </div>
            </DialogContent>
        </Dialog>
    }
}
