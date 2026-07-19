use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
}

impl BadgeVariant {
    fn class(&self) -> &'static str {
        match self {
            Self::Default => "border-transparent bg-primary text-primary-foreground shadow",
            Self::Secondary => "border-transparent bg-secondary text-secondary-foreground",
            Self::Destructive => "border-transparent bg-destructive text-destructive-foreground shadow",
            Self::Outline => "text-foreground",
        }
    }
}

#[component]
pub fn Badge(
    children: ChildrenFn,
    #[prop(default = BadgeVariant::Default)] variant: BadgeVariant,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let base = "inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2";
    let extra = class.unwrap_or("");
    let cls = move || format!("{} {} {}", base, variant.class(), extra);

    view! {
        <div data-slot="badge" class=cls()>
            {children()}
        </div>
    }
}
