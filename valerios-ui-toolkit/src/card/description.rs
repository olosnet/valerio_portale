use leptos::prelude::*;

#[component]
pub fn CardDescription(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("text-sm text-muted-foreground {}", extra);

    view! {
        <div data-slot="card-description" class=cls()>
            {children()}
        </div>
    }
}
