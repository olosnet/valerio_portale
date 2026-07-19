use leptos::prelude::*;
use super::tooltip::use_tooltip;

#[component]
pub fn TooltipContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_tooltip();
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "absolute z-50 bottom-full left-1/2 -translate-x-1/2 mb-2 rounded-md border bg-popover px-3 py-1.5 text-xs text-popover-foreground shadow-md animate-fade-in whitespace-nowrap {}",
        extra,
    );

    move || {
        if ctx.open.get() {
            view! {
                <div data-slot="tooltip-content" class=cls() role="tooltip">
                    {children()}
                </div>
            }.into_any()
        } else {
            ().into_any()
        }
    }
}
