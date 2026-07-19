use leptos::prelude::*;

#[component]
pub fn CommandGroup(
    children: ChildrenFn,
    #[prop(default = "")] heading: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!("overflow-hidden p-1 text-foreground [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-muted-foreground {}", extra);

    view! {
        <div data-slot="command-group" class=cls()>
            {move || if !heading.is_empty() {
                view! { <div cmdk-group-heading="" class="px-2 py-1.5 text-xs font-medium text-muted-foreground">{heading}</div> }.into_any()
            } else { ().into_any() }}
            {children()}
        </div>
    }
}
