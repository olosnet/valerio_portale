use leptos::prelude::*;

#[component]
pub fn H1(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! { <h1 data-slot="h1" class=format!("scroll-m-20 text-4xl font-extrabold tracking-tight text-foreground lg:text-5xl {}", extra)>{children()}</h1> }
}

#[component]
pub fn H2(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! { <h2 data-slot="h2" class=format!("scroll-m-20 text-3xl font-semibold tracking-tight text-foreground first:mt-0 {}", extra)>{children()}</h2> }
}

#[component]
pub fn H3(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! { <h3 data-slot="h3" class=format!("scroll-m-20 text-2xl font-semibold tracking-tight text-foreground {}", extra)>{children()}</h3> }
}

#[component]
pub fn H4(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! { <h4 data-slot="h4" class=format!("scroll-m-20 text-xl font-semibold tracking-tight text-foreground {}", extra)>{children()}</h4> }
}

#[component]
pub fn Lead(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! { <p data-slot="lead" class=format!("text-xl text-muted-foreground {}", extra)>{children()}</p> }
}

#[component]
pub fn Small(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! { <small data-slot="small" class=format!("text-sm font-medium leading-none text-foreground {}", extra)>{children()}</small> }
}

#[component]
pub fn Muted(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! { <p data-slot="muted" class=format!("text-sm text-muted-foreground {}", extra)>{children()}</p> }
}
