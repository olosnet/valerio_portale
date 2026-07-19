use leptos::prelude::*;

#[component]
pub fn CardFooter(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("flex items-center p-6 pt-0 {}", extra);

    view! {
        <div data-slot="card-footer" class=cls()>
            {children()}
        </div>
    }
}
