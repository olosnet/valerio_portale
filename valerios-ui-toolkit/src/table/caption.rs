use leptos::prelude::*;

#[component]
pub fn TableCaption(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("mt-4 text-sm text-muted-foreground {}", extra);

    view! {
        <caption data-slot="table-caption" class=cls()>
            {children()}
        </caption>
    }
}
