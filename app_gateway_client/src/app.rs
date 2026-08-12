use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::*;
use leptos_router::{
    ParamSegment, StaticSegment,
    components::{Redirect, Route, Router, Routes},
    hooks::use_navigate,
};

use crate::modules::auth::pages::login::Login;
use crate::modules::base::api_client::{ApiClient, set_on_session_expired};
use crate::modules::base::components::main_layout::with_layout;
use crate::modules::base::components::not_found::NotFound;
use crate::modules::groups::pages::group_detail::GroupDetail;
use crate::modules::groups::pages::groups_list::GroupsList;
use crate::modules::identity::pages::profile::Profile;
use crate::modules::oggetti_astronomici::pages::oggetti_list::OggettiList;
use crate::modules::oggetti_astronomici::pages::oggetto_detail::OggettoDetail;
use crate::modules::siti_osservativi::pages::siti_list::SitiList;
use crate::modules::siti_osservativi::pages::sito_detail::SitoDetail;
use crate::modules::users::pages::user_detail::UserDetail;
use crate::modules::users::pages::users_list::UsersList;
use crate::stores::auth_store::{provide_auth, use_auth};
use valerios_ui_toolkit::sonner::Sonner;
use valerios_ui_toolkit::theme::ThemeProvider;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let api_client = Arc::new(ApiClient::new("/api"));
    let auth = provide_auth(api_client);

    spawn_local(async move {
        auth.check_session().await;
    });

    view! {
        <Stylesheet id="leptos" href="/pkg/app_gateway_client.css"/>
        <Link rel="icon" type_="image/x-icon" href="/static/favicon.ico"/>
        <Link rel="icon" type_="image/png" sizes="16x16" href="/static/favicon-16x16.png"/>
        <Link rel="icon" type_="image/png" sizes="32x32" href="/static/favicon-32x32.png"/>
        <Link rel="icon" type_="image/png" sizes="96x96" href="/static/favicon-96x96.png"/>
        <Link rel="apple-touch-icon" sizes="57x57" href="/static/apple-icon-57x57.png"/>
        <Link rel="apple-touch-icon" sizes="60x60" href="/static/apple-icon-60x60.png"/>
        <Link rel="apple-touch-icon" sizes="72x72" href="/static/apple-icon-72x72.png"/>
        <Link rel="apple-touch-icon" sizes="76x76" href="/static/apple-icon-76x76.png"/>
        <Link rel="apple-touch-icon" sizes="114x114" href="/static/apple-icon-114x114.png"/>
        <Link rel="apple-touch-icon" sizes="120x120" href="/static/apple-icon-120x120.png"/>
        <Link rel="apple-touch-icon" sizes="144x144" href="/static/apple-icon-144x144.png"/>
        <Link rel="apple-touch-icon" sizes="152x152" href="/static/apple-icon-152x152.png"/>
        <Link rel="apple-touch-icon" sizes="180x180" href="/static/apple-icon-180x180.png"/>
        <Link rel="apple-touch-icon-precomposed" href="/static/apple-icon-precomposed.png"/>
        <Meta name="msapplication-TileColor" content="#ffffff"/>
        <Meta name="msapplication-TileImage" content="/static/ms-icon-144x144.png"/>
        <Title text="App Gateway"/>

        <ThemeProvider initial_theme="olive" default_dark=false>
            <Sonner position="bottom-right" default_duration_ms=4000 max_visible=5>
                <Router>
                    <SessionExpiredHandler/>
                    <Routes fallback=|| view! { <NotFound/> }>
                        <Route path=StaticSegment("login") view=Login/>
                        <Route path=StaticSegment("") view=ProtectedDashboard/>
                        <Route path=StaticSegment("profile") view=ProtectedProfile/>
                        <Route path=StaticSegment("siti_osservativi") view=ProtectedSitiList/>
                        <Route path=(StaticSegment("siti_osservativi"), ParamSegment("id")) view=ProtectedSitoDetail/>
                        <Route path=StaticSegment("oggetti_astronomici") view=ProtectedOggettiList/>
                        <Route path=(StaticSegment("oggetti_astronomici"), ParamSegment("id")) view=ProtectedOggettoDetail/>
                        <Route path=(StaticSegment("settings"), StaticSegment("users")) view=ProtectedUsersList/>
                        <Route path=(StaticSegment("settings"), StaticSegment("users"), ParamSegment("id")) view=ProtectedUserDetail/>
                        <Route path=(StaticSegment("settings"), StaticSegment("groups")) view=ProtectedGroupsList/>
                        <Route path=(StaticSegment("settings"), StaticSegment("groups"), ParamSegment("id")) view=ProtectedGroupDetail/>
                    </Routes>
                </Router>
            </Sonner>
        </ThemeProvider>
    }
}

#[component]
fn ProtectedDashboard() -> impl IntoView {
    let auth = use_auth();
    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }
        if !auth.is_authenticated() {
            return view! { <Redirect path="/login"/> }.into_any();
        }
        with_layout(view! { <DashboardBody/> }).into_any()
    }
}

#[component]
fn ProtectedProfile() -> impl IntoView {
    let auth = use_auth();
    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }
        if !auth.is_authenticated() {
            return view! { <Redirect path="/login"/> }.into_any();
        }
        with_layout(view! { <ProfileBody/> }).into_any()
    }
}

#[component]
fn ProtectedSitiList() -> impl IntoView {
    let auth = use_auth();
    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }
        if !auth.is_authenticated() {
            return view! { <Redirect path="/login"/> }.into_any();
        }
        with_layout(view! { <SitiList/> }).into_any()
    }
}

#[component]
fn ProtectedSitoDetail() -> impl IntoView {
    let auth = use_auth();
    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }
        if !auth.is_authenticated() {
            return view! { <Redirect path="/login"/> }.into_any();
        }
        with_layout(view! { <SitoDetail/> }).into_any()
    }
}

#[component]
fn ProtectedOggettiList() -> impl IntoView {
    let auth = use_auth();
    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }
        if !auth.is_authenticated() {
            return view! { <Redirect path="/login"/> }.into_any();
        }
        with_layout(view! { <OggettiList/> }).into_any()
    }
}

#[component]
fn ProtectedOggettoDetail() -> impl IntoView {
    let auth = use_auth();
    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }
        if !auth.is_authenticated() {
            return view! { <Redirect path="/login"/> }.into_any();
        }
        with_layout(view! { <OggettoDetail/> }).into_any()
    }
}

#[component]
fn ProtectedUsersList() -> impl IntoView {
    let auth = use_auth();
    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }
        if !auth.is_authenticated() {
            return view! { <Redirect path="/login"/> }.into_any();
        }
        with_layout(view! { <UsersList/> }).into_any()
    }
}

#[component]
fn ProtectedUserDetail() -> impl IntoView {
    let auth = use_auth();
    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }
        if !auth.is_authenticated() {
            return view! { <Redirect path="/login"/> }.into_any();
        }
        with_layout(view! { <UserDetail/> }).into_any()
    }
}

#[component]
fn ProtectedGroupsList() -> impl IntoView {
    let auth = use_auth();
    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }
        if !auth.is_authenticated() {
            return view! { <Redirect path="/login"/> }.into_any();
        }
        with_layout(view! { <GroupsList/> }).into_any()
    }
}

#[component]
fn ProtectedGroupDetail() -> impl IntoView {
    let auth = use_auth();
    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }
        if !auth.is_authenticated() {
            return view! { <Redirect path="/login"/> }.into_any();
        }
        with_layout(view! { <GroupDetail/> }).into_any()
    }
}

#[component]
fn DashboardBody() -> impl IntoView {
    let auth = use_auth();

    view! {
        <Title text="Dashboard - App Gateway"/>
        <div class="space-y-6">
            <div>
                <h2 class="text-2xl font-bold text-foreground mb-1">
                    "Benvenuto, "
                    { let auth = Arc::clone(&auth);  move || {
                        auth.get_user().as_ref().and_then(|u| u.name.clone()).unwrap_or_default()
                    }}
                </h2>
                <p class="text-muted-foreground">"Panoramica del sistema"</p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                {move || {
                    let p = auth.perms();
                    let mut cards = Vec::new();
                    if let Some(perm) = p.get("users") {
                        if perm.read {
                            cards.push(view! {
                                <a href="/settings/users" class="block bg-background rounded-lg border border-border shadow-sm p-6 hover:shadow-md transition-shadow">
                                    <h3 class="text-lg font-semibold text-foreground mb-1">"Utenti"</h3>
                                    <p class="text-sm text-muted-foreground">"Gestione utenti della piattaforma"</p>
                                </a>
                            });
                        }
                    }
                    if let Some(perm) = p.get("groups") {
                        if perm.read {
                            cards.push(view! {
                                <a href="/settings/groups" class="block bg-background rounded-lg border border-border shadow-sm p-6 hover:shadow-md transition-shadow">
                                    <h3 class="text-lg font-semibold text-foreground mb-1">"Gruppi"</h3>
                                    <p class="text-sm text-muted-foreground">"Gestione gruppi e permessi"</p>
                                </a>
                            });
                        }
                    }
                    if let Some(perm) = p.get("astronomia") {
                        if perm.read {
                            cards.push(view! {
                                <a href="/siti_osservativi" class="block bg-background rounded-lg border border-border shadow-sm p-6 hover:shadow-md transition-shadow">
                                    <h3 class="text-lg font-semibold text-foreground mb-1">"Siti Osservativi"</h3>
                                    <p class="text-sm text-muted-foreground">"Gestione siti di osservazione astronomica"</p>
                                </a>
                            });
                            cards.push(view! {
                                <a href="/oggetti_astronomici" class="block bg-background rounded-lg border border-border shadow-sm p-6 hover:shadow-md transition-shadow">
                                    <h3 class="text-lg font-semibold text-foreground mb-1">"Oggetti Astronomici"</h3>
                                    <p class="text-sm text-muted-foreground">"Catalogo oggetti celesti"</p>
                                </a>
                            });
                        }
                    }
                    cards.into_iter().collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

#[component]
fn ProfileBody() -> impl IntoView {
    view! { <Profile/> }
}

#[component]
fn SessionExpiredHandler() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    set_on_session_expired(Box::new(move || {
        auth.unset_user();
        navigate("/login", Default::default());
    }));

    view! {}
}
