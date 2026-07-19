use std::sync::Arc;
use leptos::prelude::*;

fn icon_chevron_left() -> AnyView {
    use crate::icon::Icon;
    Icon::ChevronLeft.render()
}
fn icon_chevron_right() -> AnyView {
    use crate::icon::Icon;
    Icon::ChevronRight.render()
}

#[component]
pub fn Pagination(
    children: Children,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! {
        <nav role="navigation" aria-label="pagination" data-slot="pagination" class=extra>
            {children()}
        </nav>
    }
}

#[component]
pub fn PaginationContent(
    children: Children,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! {
        <ul data-slot="pagination-content" class=format!("flex flex-row items-center gap-1 {}", extra)>
            {children()}
        </ul>
    }
}

#[component]
pub fn PaginationItem(
    children: Children,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    view! {
        <li data-slot="pagination-item" class=extra>
            {children()}
        </li>
    }
}

#[component]
pub fn PaginationLink(
    children: ChildrenFn,
    #[prop(default = false)] is_active: bool,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let base = "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-9 w-9";
    let active_cls = if is_active { " border border-border bg-accent text-accent-foreground" } else { "" };
    let cls = move || format!("{} {}{}", base, active_cls, extra);
    let handle = move |_| { if let Some(ref cb) = on_click { cb(); } };

    view! {
        <button type="button" data-slot="pagination-link" aria-current=if is_active { "page" } else { "false" } on:click=handle class=cls()>
            {children()}
        </button>
    }
}

#[component]
pub fn PaginationPrevious(
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    #[prop(default = false)] disabled: bool,
    #[prop(default = "Precedente")] text: &'static str,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let handle = move |_| { if let Some(ref cb) = on_click { cb(); } };

    view! {
        <button type="button" data-slot="pagination-previous" disabled=disabled on:click=handle
            aria_label="Go to previous page"
            class=format!("inline-flex items-center justify-center gap-1 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-9 px-3 {}", extra)>
            {icon_chevron_left()}
            <span class="hidden sm:block">{text}</span>
        </button>
    }
}

#[component]
pub fn PaginationNext(
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    #[prop(default = false)] disabled: bool,
    #[prop(default = "Successivo")] text: &'static str,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let extra = class.unwrap_or("");
    let handle = move |_| { if let Some(ref cb) = on_click { cb(); } };

    view! {
        <button type="button" data-slot="pagination-next" disabled=disabled on:click=handle
            aria_label="Go to next page"
            class=format!("inline-flex items-center justify-center gap-1 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-9 px-3 {}", extra)>
            <span class="hidden sm:block">{text}</span>
            {icon_chevron_right()}
        </button>
    }
}

#[component]
pub fn PaginationEllipsis(
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let extra = class.unwrap_or("");

    view! {
        <span aria-hidden="true" data-slot="pagination-ellipsis" class=format!("flex h-9 w-9 items-center justify-center {}", extra)>
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4">
                <circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>
            </svg>
        </span>
    }
}
