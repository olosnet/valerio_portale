#![allow(dead_code)]
use std::sync::Arc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::modules::identity::api::profile_image_url;
use crate::stores::auth_store::use_auth;

use valerios_ui_toolkit::icon::Icon;
use valerios_ui_toolkit::sidebar::{
    use_sidebar, Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel,
    SidebarHeader, SidebarMenu, SidebarMenuItem, SidebarMenuSub, SidebarMenuSubItem, SidebarRail,
};
use valerios_ui_toolkit::sheet::{Sheet, SheetContent, SheetSide};

fn menu_btn_class() -> &'static str {
    "peer/menu-button flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm ring-sidebar-ring outline-hidden transition-[width,height,padding] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 group-data-[collapsible=icon]:size-8! group-data-[collapsible=icon]:p-2! [&>svg]:size-4 [&>svg]:shrink-0 [&>span:last-child]:truncate"
}

fn sub_btn_class() -> &'static str {
    "flex h-7 min-w-0 -translate-x-px items-center gap-2 overflow-hidden rounded-md px-2 text-sidebar-foreground ring-sidebar-ring outline-hidden group-data-[collapsible=icon]:hidden hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 [&>span:last-child]:truncate w-full text-left text-sm [&>svg]:size-4 [&>svg]:shrink-0"
}

#[component]
fn AdminSection(can_see_users: Signal<bool>, can_see_groups: Signal<bool>) -> impl IntoView {
    let show = RwSignal::new(false);
    let nav = Arc::new(use_navigate());

    let on_users = { let n = nav.clone(); move |_| { n("/settings/users", Default::default()); } };
    let on_groups = { let n = nav.clone(); move |_| { n("/settings/groups", Default::default()); } };

    view! {
        <SidebarGroup>
            <SidebarGroupLabel>"Amministrazione"</SidebarGroupLabel>
            <SidebarMenu>
                <SidebarMenuItem>
                    <button on:click=move |_| show.update(|v| *v = !*v) class=menu_btn_class()>
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 7h-9"/><path d="M14 17H5"/><circle cx="17" cy="17" r="3"/><circle cx="7" cy="7" r="3"/></svg>
                        <span class="group-data-[collapsible=icon]:hidden">"Impostazioni"</span>
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="ml-auto transition-transform duration-200 shrink-0 group-data-[collapsible=icon]:hidden" class:rotate-90=move || show.get()><path d="m9 18 6-6-6-6"/></svg>
                    </button>
                    <div class:hidden=move || !show.get()>
                        <SidebarMenuSub>
                            <div class:hidden=move || !can_see_users.get()>
                                <SidebarMenuSubItem>
                                    <button on:click=on_users class=sub_btn_class()>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg><span>"Utenti"</span>
                                    </button>
                                </SidebarMenuSubItem>
                            </div>
                            <div class:hidden=move || !can_see_groups.get()>
                                <SidebarMenuSubItem>
                                    <button on:click=on_groups class=sub_btn_class()>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/></svg><span>"Gruppi"</span>
                                    </button>
                                </SidebarMenuSubItem>
                            </div>
                        </SidebarMenuSub>
                    </div>
                </SidebarMenuItem>
            </SidebarMenu>
        </SidebarGroup>
    }
}

#[component]
fn DesktopSidebar(auth: crate::stores::auth_store::AuthContext) -> impl IntoView {
    let nav = Arc::new(use_navigate());
    let can_see_users = auth.can_read_signal("users");
    let can_see_groups = auth.can_read_signal("groups");
    let has_admin = Signal::derive(move || can_see_users.get() || can_see_groups.get());
    let n = nav.clone();
    let a = auth.clone();
    let n_profile = nav.clone();
    let n_logout = nav.clone();

    fn avatar_view(user: Option<&crate::modules::identity::models::UserIdentity>) -> AnyView {
        let img = user.map(|u| u.profile_image.clone()).unwrap_or_default();
        let url = profile_image_url("/api", &img);
        if url.is_empty() {
            let initial = user.and_then(|u| u.name.as_ref().and_then(|n| n.chars().next().map(|c| c.to_uppercase().to_string()))).unwrap_or_default();
            view! {
                <div class="flex size-8 items-center justify-center rounded-md bg-sidebar-accent text-sidebar-accent-foreground font-medium text-sm shrink-0">{initial}</div>
            }.into_any()
        } else {
            view! {
                <img src=url class="size-8 rounded-md object-cover shrink-0" alt="Avatar" />
            }.into_any()
        }
    }

    view! {
        <Sidebar collapsible="icon">
            <SidebarHeader>
                <div class="flex items-center gap-2 px-2 py-1">
                    <img src="/static/logo.svg" class="size-8 shrink-0" alt="Vita" />
                    <div class="grid flex-1 text-left text-sm leading-tight overflow-hidden group-data-[collapsible=icon]:hidden"><span class="truncate font-medium">"Vita"</span></div>
                </div>
            </SidebarHeader>
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>"Navigazione"</SidebarGroupLabel>
                    <SidebarMenu>
                        <SidebarMenuItem>
                            <button on:click=move |_| { let _ = n("/", Default::default()); } class=menu_btn_class()>
                                {Icon::LayoutDashboard.render()}<span>"Dashboard"</span>
                            </button>
                        </SidebarMenuItem>
                    </SidebarMenu>
                </SidebarGroup>
                {move || if has_admin.get() {
                    view! { <AdminSection can_see_users=can_see_users can_see_groups=can_see_groups /> }.into_any()
                } else { ().into_any() }}
            </SidebarContent>
            <SidebarFooter>
                <SidebarMenu>
                    <SidebarMenuItem>
                        <div class="flex items-center gap-1">
                            <button on:click=move |_| { let _ = n_profile("/profile", Default::default()); }
                                class="peer/menu-button flex flex-1 items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm ring-sidebar-ring outline-hidden transition-[width,height,padding] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground group-data-[collapsible=icon]:size-8! group-data-[collapsible=icon]:p-2! [&>svg]:size-4 [&>svg]:shrink-0 [&>span:last-child]:truncate">
                                {move || avatar_view(auth.user.get().as_ref())}
                                <div class="grid flex-1 text-left text-sm leading-tight overflow-hidden group-data-[collapsible=icon]:hidden">
                                    <span class="truncate font-medium">{move || auth.user.get().as_ref().and_then(|u| u.name.clone()).unwrap_or_default()}</span>
                                    <span class="truncate text-xs text-sidebar-foreground/60">{move || auth.user.get().as_ref().and_then(|u| u.email.clone()).unwrap_or_default()}</span>
                                </div>
                            </button>
                            <button on:click=move |_| {
                                let x = a.clone();
                                spawn_local(async move { x.logout().await; });
                                let _ = n_logout("/login", Default::default());
                            }
                                class="inline-flex items-center justify-center rounded-md text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground h-8 w-8 shrink-0 transition-colors group-data-[collapsible=icon]:hidden"
                                title="Esci"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" x2="9" y1="12" y2="12"/></svg>
                            </button>
                        </div>
                    </SidebarMenuItem>
                </SidebarMenu>
            </SidebarFooter>
            <SidebarRail/>
        </Sidebar>
    }
}

#[component]
fn MobileSidebar(auth: crate::stores::auth_store::AuthContext) -> impl IntoView {
    let nav = Arc::new(use_navigate());
    let sctx = use_sidebar();
    let can_see_users = auth.can_read_signal("users");
    let can_see_groups = auth.can_read_signal("groups");
    let has_admin = Signal::derive(move || can_see_users.get() || can_see_groups.get());
    let n = nav.clone();
    let a = auth.clone();
    let n_profile = nav.clone();
    let n_logout = nav.clone();

    view! {
        <Sheet open=sctx.open_mobile side=SheetSide::Left>
            <SheetContent>
                <Sidebar variant="mobile">
                    <SidebarHeader>
                        <div class="flex items-center gap-2 px-2 py-1">
                            <img src="/static/logo.svg" class="size-8 shrink-0" alt="Vita" />
                            <div class="grid flex-1 text-left text-sm leading-tight overflow-hidden"><span class="truncate font-medium">"Vita"</span></div>
                        </div>
                    </SidebarHeader>
                    <SidebarContent>
                        <SidebarGroup>
                            <SidebarGroupLabel>"Navigazione"</SidebarGroupLabel>
                            <SidebarMenu>
                                <SidebarMenuItem>
                                    <button on:click=move |_| { let _ = n("/", Default::default()); sctx.open_mobile.set(false); } class=menu_btn_class()>
                                        {Icon::LayoutDashboard.render()}<span>"Dashboard"</span>
                                    </button>
                                </SidebarMenuItem>
                            </SidebarMenu>
                        </SidebarGroup>
                        {move || if has_admin.get() {
                            view! { <AdminSection can_see_users=can_see_users can_see_groups=can_see_groups /> }.into_any()
                        } else { ().into_any() }}
                    </SidebarContent>
                    <SidebarFooter>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <div class="flex items-center gap-1">
                                    <button on:click=move |_| { let _ = n_profile("/profile", Default::default()); sctx.open_mobile.set(false); }
                                        class="flex flex-1 items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm ring-sidebar-ring outline-hidden transition-[width,height,padding] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground [&>svg]:size-4 [&>svg]:shrink-0 [&>span:last-child]:truncate">
                                        {move || {
                                            let user = auth.user.get();
                                            let img = user.as_ref().map(|u| u.profile_image.clone()).unwrap_or_default();
                                            let url = profile_image_url("/api", &img);
                                            if url.is_empty() {
                                                let initial = user.as_ref().and_then(|u| u.name.as_ref().and_then(|n| n.chars().next().map(|c| c.to_uppercase().to_string()))).unwrap_or_default();
                                                view! {
                                                    <div class="flex size-8 items-center justify-center rounded-md bg-sidebar-accent text-sidebar-accent-foreground font-medium text-sm shrink-0">{initial}</div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <img src=url class="size-8 rounded-md object-cover shrink-0" alt="Avatar" />
                                                }.into_any()
                                            }
                                        }}
                                        <div class="grid flex-1 text-left text-sm leading-tight overflow-hidden">
                                            <span class="truncate font-medium">{move || auth.user.get().as_ref().and_then(|u| u.name.clone()).unwrap_or_default()}</span>
                                            <span class="truncate text-xs text-sidebar-foreground/60">{move || auth.user.get().as_ref().and_then(|u| u.email.clone()).unwrap_or_default()}</span>
                                        </div>
                                    </button>
                                    <button on:click=move |_| {
                                        let x = a.clone();
                                        spawn_local(async move { x.logout().await; });
                                        let _ = n_logout("/login", Default::default());
                                        sctx.open_mobile.set(false);
                                    }
                                        class="inline-flex items-center justify-center rounded-md text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground h-8 w-8 shrink-0 transition-colors"
                                        title="Esci"
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-4"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" x2="9" y1="12" y2="12"/></svg>
                                    </button>
                                </div>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarFooter>
                </Sidebar>
            </SheetContent>
        </Sheet>
    }
}

#[component]
pub fn AppSidebar() -> impl IntoView {
    let auth = use_auth();

    view! {
        <div class="hidden lg:block">
            <DesktopSidebar auth=auth.clone()/>
        </div>
        <div class="lg:hidden">
            <MobileSidebar auth=auth.clone()/>
        </div>
    }
}
