use std::sync::Arc;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ToggleVariant {
    Default,
    Outline,
}

impl ToggleVariant {
    fn class(&self) -> &'static str {
        match self {
            Self::Default => "bg-transparent",
            Self::Outline => "border border-input bg-transparent shadow-sm hover:bg-accent hover:text-accent-foreground",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ToggleSize {
    Default,
    Sm,
    Lg,
}

impl ToggleSize {
    fn class(&self) -> &'static str {
        match self {
            Self::Default => "h-9 px-3",
            Self::Sm => "h-8 px-2",
            Self::Lg => "h-10 px-3",
        }
    }
}

#[component]
pub fn Toggle(
    children: ChildrenFn,
    pressed: RwSignal<bool>,
    #[prop(optional)] on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    #[prop(default = false)] disabled: bool,
    #[prop(default = ToggleVariant::Default)] variant: ToggleVariant,
    #[prop(default = ToggleSize::Default)] size: ToggleSize,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let base = "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium ring-offset-background transition-colors hover:bg-muted hover:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=on]:bg-accent data-[state=on]:text-accent-foreground [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0";
    let extra = class.unwrap_or("");
    let cls = move || format!("{} {} {} {}", base, variant.class(), size.class(), extra);

    let handle_click = move |_| {
        if disabled { return; }
        let new_val = !pressed.get();
        pressed.set(new_val);
        if let Some(ref cb) = on_change {
            cb(new_val);
        }
    };

    view! {
        <button
            type="button"
            disabled=disabled
            data-state=move || if pressed.get() { "on" } else { "off" }
            on:click=handle_click
            class=cls()
        >
            {children()}
        </button>
    }
}
