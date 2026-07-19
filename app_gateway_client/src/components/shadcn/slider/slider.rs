#![allow(dead_code)]
use leptos::prelude::*;

#[component]
pub fn Slider(
    value: RwSignal<u8>,
    #[prop(default = 0)] min: u8,
    #[prop(default = 100)] max: u8,
    #[prop(default = 1)] step: u8,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "relative flex w-full touch-none select-none items-center [&>input]:h-2 [&>input]:w-full [&>input]:cursor-pointer [&>input]:appearance-none [&>input]:rounded-full [&>input]:bg-secondary [&>input]:outline-none [&>input]:[&::-webkit-slider-thumb]:appearance-none [&>input]:[&::-webkit-slider-thumb]:h-5 [&>input]:[&::-webkit-slider-thumb]:w-5 [&>input]:[&::-webkit-slider-thumb]:rounded-full [&>input]:[&::-webkit-slider-thumb]:bg-primary [&>input]:[&::-webkit-slider-thumb]:shadow {}",
        extra,
    );

    let on_input = move |ev: leptos::ev::Event| {
        let val: u8 = event_target_value(&ev).parse().unwrap_or(0);
        value.set(val);
    };

    view! {
        <div data-slot="slider" class=cls()>
            <input
                type="range"
                min=min.to_string()
                max=max.to_string()
                step=step.to_string()
                prop:value=move || value.get().to_string()
                on:input=on_input
            />
        </div>
    }
}
