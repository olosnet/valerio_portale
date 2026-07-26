use leptos::prelude::*;
use super::collapsible::use_collapsible;

#[component]
pub fn CollapsibleContent(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_collapsible();
    let extra = class.unwrap_or("");

    let base = "grid transition-[grid-template-rows] duration-300";

    move || {
        let is_open = ctx.open.get();
        view! {
            <div
                data-state=if is_open { "open" } else { "closed" }
                data-slot="collapsible-content"
                class=format!("{} grid-rows-[0fr] data-[state=open]:grid-rows-[1fr] {}", base, extra)
            >
                <div class="overflow-hidden">
                    {children()}
                </div>
            </div>
        }.into_any()
    }
}
