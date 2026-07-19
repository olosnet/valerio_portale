use leptos::prelude::*;

const SCROLLBAR_STYLE: &str = ".scroll-area-custom::-webkit-scrollbar{width:6px;height:6px}.scroll-area-custom::-webkit-scrollbar-track{background:transparent}.scroll-area-custom::-webkit-scrollbar-thumb{background:var(--border);border-radius:3px}.scroll-area-custom::-webkit-scrollbar-thumb:hover{background:var(--muted-foreground)}";

#[component]
pub fn ScrollArea(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("scroll-area-custom overflow-auto {}", extra);

    view! {
        <div data-slot="scroll-area" class=cls()>
            <style>{SCROLLBAR_STYLE}</style>
            {children()}
        </div>
    }
}
