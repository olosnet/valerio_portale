use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::stores::auth_store::use_auth;

#[component]
pub fn Sidebar() -> impl IntoView {
    let auth = use_auth();
    let navigate = Arc::new(use_navigate());
    let show_impostazioni = RwSignal::new(false);

    let can_see_users = auth.can_read_signal("users");
    let can_see_groups = auth.can_read_signal("groups");
    let has_admin = Signal::derive(move || can_see_users.get() || can_see_groups.get());

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
        <aside class="w-64 min-h-screen bg-background border-r border-border flex flex-col">
            <div class="p-4 border-b border-border">
                <h1 class="text-lg font-bold text-foreground">"App Gateway"</h1>
            </div>

            <nav class="flex-1 p-2 space-y-1">
                <button on:click={ let n = navigate.clone(); move |_| { n("/", Default::default()); } }
                    class="w-full text-left px-3 py-2 rounded-md text-sm text-foreground hover:bg-secondary transition-colors"
                >"🏠  Dashboard"</button>

                <button on:click={ let n = navigate.clone(); move |_| { n("/profile", Default::default()); } }
                    class="w-full text-left px-3 py-2 rounded-md text-sm text-foreground hover:bg-secondary transition-colors"
                >"👤  Il mio profilo"</button>

                {move || {
                    if has_admin.get() {
                        view! {
                            <div class="pt-2">
                                <button on:click=move |_| show_impostazioni.update(|v| *v = !*v)
                                    class="w-full text-left px-3 py-2 rounded-md text-sm font-medium text-foreground hover:bg-secondary transition-colors"
                                >"⚙️  Impostazioni"</button>
                                {{
                                    let inner_nav = navigate.clone();
                                    move || {
                                        if show_impostazioni.get() {
                                            let show_users = can_see_users.get();
                                            let show_groups = can_see_groups.get();
                                            if show_users || show_groups {
                                                view! {
                                                    <div class="ml-4 mt-1 space-y-1">
                                                        {if show_users {
                                                            let n = inner_nav.clone();
                                                            Some(view! {
                                                                <button on:click=move |_| { n("/settings/users", Default::default()); }
                                                                    class="w-full text-left px-3 py-2 rounded-md text-sm text-muted-foreground hover:bg-secondary hover:text-foreground transition-colors"
                                                                >"👥  Utenti"</button>
                                                            })
                                                        } else { None }}
                                                        {if show_groups {
                                                            let n = inner_nav.clone();
                                                            Some(view! {
                                                                <button on:click=move |_| { n("/settings/groups", Default::default()); }
                                                                    class="w-full text-left px-3 py-2 rounded-md text-sm text-muted-foreground hover:bg-secondary hover:text-foreground transition-colors"
                                                                >"👥  Gruppi"</button>
                                                            })
                                                        } else { None }}
                                                    </div>
                                                }.into_any()
                                            } else { ().into_any() }
                                        } else { ().into_any() }
                                    }
                                }}
                            </div>
                        }.into_any()
                    } else { ().into_any() }
                }}
            </nav>

            <div class="p-4 border-t border-border">
                <div class="flex items-center justify-between">
                    <div class="text-sm text-foreground">
                        {move || {
                            auth.user.get().as_ref().and_then(|u| u.name.clone()).unwrap_or_default()
                        }}
                    </div>
                    <button on:click=do_logout
                        class="text-xs text-muted-foreground hover:text-destructive transition-colors"
                    >"Esci"</button>
                </div>
            </div>
        </aside>
    }
}
