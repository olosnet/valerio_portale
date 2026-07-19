use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::stores::auth_store::use_auth;

fn menu_button_classes() -> &'static str {
    "flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm \
     ring-sidebar-ring outline-hidden transition-[width,height,padding] \
     hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 \
     active:bg-sidebar-accent active:text-sidebar-accent-foreground \
     [&>svg]:size-4 [&>svg]:shrink-0 [&>span:last-child]:truncate"
}

fn sub_button_classes() -> &'static str {
    "flex h-7 min-w-0 -translate-x-px items-center gap-2 overflow-hidden rounded-md px-2 \
     text-sidebar-foreground ring-sidebar-ring outline-hidden \
     hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 \
     active:bg-sidebar-accent active:text-sidebar-accent-foreground \
     [&>span:last-child]:truncate w-full text-left text-sm [&>svg]:size-4 [&>svg]:shrink-0"
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let auth = use_auth();
    let navigate = Arc::new(use_navigate());
    let show_impostazioni = RwSignal::new(false);

    let can_see_users = auth.can_read_signal("users");
    let can_see_groups = auth.can_read_signal("groups");
    let has_admin = Signal::derive(move || can_see_users.get() || can_see_groups.get());

    let user_name = Signal::derive(move || {
        auth.user.get().as_ref().and_then(|u| u.name.clone()).unwrap_or_default()
    });

    let user_email = Signal::derive(move || {
        auth.user.get().as_ref().and_then(|u| u.email.clone()).unwrap_or_default()
    });

    let user_initials = Signal::derive(move || {
        auth.user.get().as_ref().and_then(|u| {
            u.name.as_ref().and_then(|n| n.chars().next()).map(|c| c.to_uppercase().to_string())
        }).unwrap_or_default()
    });

    let do_logout = {
        let auth = auth.clone();
        let navigate = navigate.clone();
        move |_| {
            let auth = auth.clone();
            let navigate = navigate.clone();
            spawn_local(async move {
                auth.logout().await;
                (navigate)("/login", Default::default());
            });
        }
    };

    view! {
        <aside data-slot="sidebar"
            class="flex h-full w-64 flex-col bg-sidebar text-sidebar-foreground border-r border-sidebar-border"
        >
            <div data-slot="sidebar-header" class="flex flex-col gap-2 p-2">
                <div class="flex items-center gap-2 px-2 py-1">
                    <div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground font-bold text-sm">
                        "AG"
                    </div>
                    <div class="grid flex-1 text-left text-sm leading-tight">
                        <span class="truncate font-medium">"App Gateway"</span>
                    </div>
                </div>
            </div>

            <div data-slot="sidebar-content" class="flex min-h-0 flex-1 flex-col gap-0 overflow-auto">
                <div data-slot="sidebar-group" class="relative flex w-full min-w-0 flex-col p-2">
                    <div data-slot="sidebar-group-label"
                        class="flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-sidebar-foreground/70"
                    >
                        "Navigazione"
                    </div>
                    <ul data-slot="sidebar-menu" class="flex w-full min-w-0 flex-col gap-0">
                        <li data-slot="sidebar-menu-item" class="group/menu-item relative">
                            <button on:click={ let n = navigate.clone(); move |_| { n("/", Default::default()); } }
                                class=menu_button_classes()
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="7" height="9" x="3" y="3" rx="1"/><rect width="7" height="5" x="14" y="3" rx="1"/><rect width="7" height="9" x="14" y="12" rx="1"/><rect width="7" height="5" x="3" y="16" rx="1"/></svg>
                                <span>"Dashboard"</span>
                            </button>
                        </li>
                        <li data-slot="sidebar-menu-item" class="group/menu-item relative">
                            <button on:click={ let n = navigate.clone(); move |_| { n("/profile", Default::default()); } }
                                class=menu_button_classes()
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                                <span>"Il mio profilo"</span>
                            </button>
                        </li>
                    </ul>
                </div>

                {move || {
                    let nav = navigate.clone();
                    if has_admin.get() {
                        view! {
                            <div data-slot="sidebar-group"
                                class="relative flex w-full min-w-0 flex-col p-2"
                            >
                                <div data-slot="sidebar-group-label"
                                    class="flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-sidebar-foreground/70"
                                >
                                    "Amministrazione"
                                </div>
                                <ul data-slot="sidebar-menu" class="flex w-full min-w-0 flex-col gap-0">
                                    <li data-slot="sidebar-menu-item" class="group/menu-item relative">
                                        <button on:click=move |_| show_impostazioni.update(|v| *v = !*v)
                                            class=menu_button_classes()
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 7h-9"/><path d="M14 17H5"/><circle cx="17" cy="17" r="3"/><circle cx="7" cy="7" r="3"/></svg>
                                            <span>"Impostazioni"</span>
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                                                class="ml-auto transition-transform duration-200"
                                                class:rotate-90=move || show_impostazioni.get()
                                            ><path d="m9 18 6-6-6-6"/></svg>
                                        </button>
                                        {move || {
                                            let sub_nav = nav.clone();
                                            if show_impostazioni.get() {
                                                let show_users = can_see_users.get();
                                                let show_groups = can_see_groups.get();
                                                if show_users || show_groups {
                                                    view! {
                                                        <ul data-slot="sidebar-menu-sub"
                                                            class="mx-3.5 flex min-w-0 translate-x-px flex-col gap-1 border-l border-sidebar-border px-2.5 py-0.5"
                                                        >
                                                            {if show_users {
                                                                let n = sub_nav.clone();
                                                                Some(view! {
                                                                    <li data-slot="sidebar-menu-sub-item" class="group/menu-sub-item relative">
                                                                        <button on:click=move |_| { n("/settings/users", Default::default()); }
                                                                            class=sub_button_classes()
                                                                        >
                                                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
                                                                            <span>"Utenti"</span>
                                                                        </button>
                                                                    </li>
                                                                })
                                                            } else { None }}
                                                            {if show_groups {
                                                                let n = sub_nav.clone();
                                                                Some(view! {
                                                                    <li data-slot="sidebar-menu-sub-item" class="group/menu-sub-item relative">
                                                                        <button on:click=move |_| { n("/settings/groups", Default::default()); }
                                                                            class=sub_button_classes()
                                                                        >
                                                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/></svg>
                                                                            <span>"Gruppi"</span>
                                                                        </button>
                                                                    </li>
                                                                })
                                                            } else { None }}
                                                        </ul>
                                                    }.into_any()
                                                } else { ().into_any() }
                                            } else { ().into_any() }
                                        }}
                                    </li>
                                </ul>
                            </div>
                        }.into_any()
                    } else { ().into_any() }
                }}
            </div>

            <div data-slot="sidebar-footer" class="flex flex-col gap-2 p-2 border-t border-sidebar-border">
                <ul data-slot="sidebar-menu" class="flex w-full min-w-0 flex-col gap-0">
                    <li data-slot="sidebar-menu-item" class="group/menu-item relative">
                        <button on:click=do_logout
                            class=menu_button_classes()
                        >
                            <div class="flex size-8 items-center justify-center rounded-md bg-sidebar-accent text-sidebar-accent-foreground font-medium text-sm shrink-0">
                                {move || user_initials.get()}
                            </div>
                            <div class="grid flex-1 text-left text-sm leading-tight">
                                <span class="truncate font-medium">{move || user_name.get()}</span>
                                <span class="truncate text-xs text-sidebar-foreground/60">{move || user_email.get()}</span>
                            </div>
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-muted-foreground shrink-0"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" x2="9" y1="12" y2="12"/></svg>
                        </button>
                    </li>
                </ul>
            </div>
        </aside>
    }
}
