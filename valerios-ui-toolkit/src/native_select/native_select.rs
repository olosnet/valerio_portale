use leptos::prelude::*;

#[component]
pub fn NativeSelect(
    children: ChildrenFn,
    value: RwSignal<String>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let cls = move || format!(
        "flex h-10 w-full items-center justify-between rounded-md border border-input bg-background text-foreground px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1 {}",
        extra,
    );

    let on_change = move |ev: leptos::ev::Event| {
        value.set(event_target_value(&ev));
    };

    view! {
        <select data-slot="native-select" class=cls() on:change=on_change>
            {children()}
        </select>
    }
}
