use std::sync::Arc;

use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

impl ButtonVariant {
    fn class(&self) -> &'static str {
        match self {
            Self::Default => "bg-primary text-primary-foreground shadow hover:bg-primary/90",
            Self::Destructive => "bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90",
            Self::Outline => "border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground",
            Self::Secondary => "bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80",
            Self::Ghost => "hover:bg-accent hover:text-accent-foreground",
            Self::Link => "text-primary underline-offset-4 hover:underline",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonSize {
    Default,
    Sm,
    Lg,
    Icon,
}

impl ButtonSize {
    fn class(&self) -> &'static str {
        match self {
            Self::Default => "h-9 px-4 py-2",
            Self::Sm => "h-8 rounded-md px-3 text-xs",
            Self::Lg => "h-10 rounded-md px-8",
            Self::Icon => "h-9 w-9",
        }
    }
}

#[component]
pub fn Button(
    children: ChildrenFn,
    #[prop(default = ButtonVariant::Default)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Default)] size: ButtonSize,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let base = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0";
    let extra = class.unwrap_or("");
    let cls = move || format!("{} {} {} {}", base, variant.class(), size.class(), extra);

    let handle_click = move |_| {
        if let Some(ref cb) = on_click {
            cb();
        }
    };

    view! {
        <button
            on:click=handle_click
            disabled=disabled
            class=cls()
            type="button"
        >
            {children()}
        </button>
    }
}
