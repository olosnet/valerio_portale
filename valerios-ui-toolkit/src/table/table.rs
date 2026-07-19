use leptos::prelude::*;

#[component]
pub fn Table(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("w-full caption-bottom text-sm text-foreground {}", extra);

    view! {
        <table data-slot="table" class=cls()>
            {children()}
        </table>
    }
}
