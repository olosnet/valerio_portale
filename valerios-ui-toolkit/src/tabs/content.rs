use leptos::prelude::*;
use super::tabs::use_tabs;

#[component]
pub fn TabsContent(
    children: ChildrenFn,
    value: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let ctx = use_tabs();
    let extra = class.unwrap_or("");
    let cls = move || format!("mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 {}", extra);

    move || {
        if ctx.value.get() == value {
            view! {
                <div role="tabpanel" data-slot="tabs-content" class=cls()>
                    {children()}
                </div>
            }.into_any()
        } else {
            ().into_any()
        }
    }
}
