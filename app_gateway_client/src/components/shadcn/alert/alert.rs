#![allow(dead_code)]
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum AlertVariant {
    Default,
    Destructive,
}

impl AlertVariant {
    fn class(&self) -> &'static str {
        match self {
            Self::Default => "bg-background text-foreground",
            Self::Destructive => "border-destructive/50 text-destructive dark:border-destructive [&>svg]:text-destructive",
        }
    }
}

#[component]
pub fn Alert(
    children: ChildrenFn,
    #[prop(default = AlertVariant::Default)] variant: AlertVariant,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let base = "relative w-full rounded-lg border p-4 [&>svg~*]:pl-7 [&>svg+div]:translate-y-[-3px] [&>svg]:absolute [&>svg]:left-4 [&>svg]:top-4 [&>svg]:text-foreground";
    let extra = class.unwrap_or("");
    let cls = move || format!("{} {} {}", base, variant.class(), extra);

    view! {
        <div role="alert" data-slot="alert" class=cls()>
            {children()}
        </div>
    }
}
