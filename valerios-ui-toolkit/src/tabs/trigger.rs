use leptos::prelude::*;
use super::tabs::use_tabs;

#[component]
pub fn TabsTrigger(
    children: ChildrenFn,
    value: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_tabs();
    let extra = class.unwrap_or("");
    let cls = move || {
        let active = ctx.value.get() == value;
        format!(
            "inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm {} {}",
            extra,
            if active { "bg-background text-foreground shadow-sm" } else { "" },
        )
    };

    let handle_click = move |_| {
        ctx.value.set(value.to_string());
    };

    view! {
        <button
            type="button"
            role="tab"
            data-state=move || if ctx.value.get() == value { "active" } else { "inactive" }
            on:click=handle_click
            class=cls()
        >
            {children()}
        </button>
    }
}
