use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::NavigateOptions;
use leptos_router::components::Redirect;
use leptos_router::hooks::{use_navigate, use_query_map};
use valerios_ui_toolkit::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
use valerios_ui_toolkit::password_input::PasswordInput;

use crate::modules::auth::api as auth_api;
use crate::modules::auth::models::OAuth2ProviderInfo;
use crate::modules::base::toast_utils::{toast_error, use_toast_ctx};
use crate::stores::auth_store::use_auth;

/// Icone SVG inline per i provider OAuth2 built-in. I provider custom
/// (es. kanidm) non hanno icona: il pulsante mostra solo il testo.
fn provider_icon_svg(name: &str) -> Option<&'static str> {
    match name {
        "google" => Some(
            r##"<svg viewBox="0 0 24 24" class="h-4 w-4" aria-hidden="true"><path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/><path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/><path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/><path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/></svg>"##,
        ),
        "github" => Some(
            r##"<svg viewBox="0 0 24 24" class="h-4 w-4 fill-current" aria-hidden="true"><path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></svg>"##,
        ),
        "microsoft" => Some(
            r##"<svg viewBox="0 0 24 24" class="h-4 w-4" aria-hidden="true"><rect width="11" height="11" fill="#F25022"/><rect x="13" width="11" height="11" fill="#7FBA00"/><rect y="13" width="11" height="11" fill="#00A4EF"/><rect x="13" y="13" width="11" height="11" fill="#FFB900"/></svg>"##,
        ),
        "apple" => Some(
            r##"<svg viewBox="0 0 24 24" class="h-4 w-4 fill-current" aria-hidden="true"><path d="M16.365 1.43c0 1.14-.493 2.27-1.177 3.08-.744.9-1.99 1.57-2.987 1.57-.12 0-.23-.02-.3-.03-.01-.06-.04-.22-.04-.39 0-1.15.572-2.27 1.206-2.98.804-.94 2.142-1.64 3.248-1.68.03.13.05.28.05.43zm4.565 15.71c-.03.07-.463 1.58-1.518 3.12-.945 1.34-1.94 2.71-3.43 2.71-1.517 0-1.9-.88-3.63-.88-1.698 0-2.302.91-3.67.91-1.377 0-2.332-1.26-3.428-2.8-1.287-1.82-2.323-4.63-2.323-7.28 0-4.28 2.797-6.55 5.552-6.55 1.448 0 2.675.95 3.6.95.865 0 2.222-1.01 3.902-1.01.613 0 2.886.06 4.374 2.19-.13.09-2.383 1.37-2.383 4.19 0 3.26 2.854 4.42 2.954 4.45z"/></svg>"##,
        ),
        "facebook" => Some(
            r##"<svg viewBox="0 0 24 24" class="h-4 w-4 fill-current" aria-hidden="true"><path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z"/></svg>"##,
        ),
        _ => None,
    }
}

fn provider_label(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Mappa i codici di errore OAuth2 del server (query `?oauth2_error=`) in
/// messaggi leggibili dall'utente.
fn oauth2_error_message(code: &str) -> String {
    match code {
        "BE_USER_NOT_FOUND" => "La creazione dell'account tramite provider non è consentita: \
             contatta l'amministratore per ottenere un account."
            .to_string(),
        "BE_PKCE_NOT_FOUND" | "BE_STATE_MISMATCH" => {
            "La richiesta di login è scaduta o non valida: riprova.".to_string()
        }
        _ => format!("Errore durante l'accesso con il provider ({code})."),
    }
}

#[component]
fn OAuth2ProviderButton(provider: OAuth2ProviderInfo) -> impl IntoView {
    let login_path = provider.login_path.clone();
    let icon = provider_icon_svg(&provider.name);
    let label = provider_label(&provider.name);

    view! {
        <button
            on:click=move |_| {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href(&login_path);
                }
            }
            class="w-full flex items-center justify-center gap-2 px-4 py-2 rounded-md border border-border bg-background text-sm font-medium text-foreground hover:bg-muted transition-colors"
        >
            {icon.map(|svg| {
                view! { <span inner_html=svg></span> }.into_any()
            })}
            <span>"Continua con " {label}</span>
        </button>
    }
}

#[component]
pub fn Login() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let toast = use_toast_ctx();

    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let toast = toast.clone();

    let oauth2_providers = RwSignal::new(Vec::<OAuth2ProviderInfo>::new());
    let oauth2_loaded = RwSignal::new(false);
    let api_client = auth.get_api_client();

    // Errore riportato dal callback OAuth2 (es. creazione utente disabilitata):
    // banner inline + toast, poi pulisce l'URL.
    let oauth2_error = use_query_map();
    let oauth2_error_code = RwSignal::new(None::<String>);
    let toast_oauth2 = toast.clone();
    let navigate_oauth2 = navigate.clone();
    let oauth2_error_handled = RwSignal::new(false);
    Effect::new(move |_| {
        if oauth2_error_handled.get_untracked() {
            return;
        }
        if let Some(code) = oauth2_error.get().get("oauth2_error") {
            oauth2_error_handled.set(true);
            oauth2_error_code.set(Some(code.clone()));
            toast_error(&toast_oauth2, &oauth2_error_message(&code));
            let _ = navigate_oauth2(
                "/login",
                NavigateOptions {
                    replace: true,
                    ..NavigateOptions::default()
                },
            );
        }
    });

    spawn_local(async move {
        match auth_api::oauth2_providers(&api_client).await {
            Ok(Some(response)) => oauth2_providers.set(response.providers),
            Ok(None) => {} // OAuth2 disabilitato: nessun provider mostrato
            Err(_) => {}   // Errore di rete: il form classico resta disponibile
        }
        oauth2_loaded.set(true);
    });

    move || {
        if !auth.initial_check_done() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }
            .into_any();
        }

        if auth.is_authenticated() {
            return view! { <Redirect path="/"/> }.into_any();
        }

        let toast_submit = toast.clone();
        let on_submit = {
            let auth = auth.clone();
            let navigate = navigate.clone();
            move |_| {
                let auth = auth.clone();
                let navigate = navigate.clone();
                let toast = toast_submit.clone();
                spawn_local(async move {
                    loading.set(true);

                    match auth
                        .login(&username.get_untracked(), &password.get_untracked())
                        .await
                    {
                        Ok(()) => {
                            let _ = navigate("/", Default::default());
                        }
                        Err(e) => {
                            toast_error(&toast, &e);
                        }
                    }

                    loading.set(false);
                });
            }
        };

        let show_oauth2 = move || oauth2_loaded.get() && !oauth2_providers.get().is_empty();

        view! {
            <Title text="Login - Vita"/>
            <div class="min-h-screen flex items-center justify-center bg-secondary">
                <div class="w-full max-w-sm mx-4">
                    <div class="bg-background rounded-lg border border-border shadow-sm p-6">
                        <div class="text-center mb-6">
                            <img src="/static/logo.svg" class="mx-auto h-16 w-16 mb-3" alt="Vita" />
                            <h1 class="text-2xl font-bold text-foreground">"Vita"</h1>
                            <p class="text-sm text-muted-foreground mt-1">"Accedi per continuare"</p>
                        </div>

                        <Show when=move || oauth2_error_code.get().is_some()>
                            <div class="mb-4">
                                <Alert variant=AlertVariant::Destructive>
                                    <AlertTitle>"Accesso non riuscito"</AlertTitle>
                                    <AlertDescription>
                                        {move || oauth2_error_code
                                            .get()
                                            .as_deref()
                                            .map(oauth2_error_message)
                                            .unwrap_or_default()}
                                    </AlertDescription>
                                </Alert>
                            </div>
                        </Show>

                        <div class="space-y-4">
                            <div>
                                <label for="username" class="block text-sm font-medium text-foreground mb-1">
                                    "Email"
                                </label>
                                <input
                                    id="username"
                                    type="text"
                                    prop:value=username
                                    on:input=move |e| username.set(event_target_value(&e))
                                    class="w-full px-3 py-2 rounded-md border border-input bg-background text-foreground text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                                    placeholder="nome@esempio.it"
                                    disabled=move || loading.get()
                                />
                            </div>

                            <div>
                                <label for="password" class="block text-sm font-medium text-foreground mb-1">
                                    "Password"
                                </label>
                                <PasswordInput value=password id="password" placeholder="••••••••" disabled=loading.into() />
                            </div>

                            <button
                                on:click=on_submit
                                disabled=move || loading.get()
                                class="w-full px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors"
                            >
                                {move || if loading.get() { "Accesso in corso..." } else { "Accedi" }}
                            </button>
                        </div>

                        <Show when=show_oauth2>
                            <div class="mt-6">
                                <div class="relative">
                                    <div class="absolute inset-0 flex items-center">
                                        <div class="w-full border-t border-border"></div>
                                    </div>
                                    <div class="relative flex justify-center text-xs">
                                        <span class="bg-background px-2 text-muted-foreground">"oppure accedi con"</span>
                                    </div>
                                </div>
                                <div class="mt-4 space-y-2">
                                    {move || oauth2_providers
                                        .get()
                                        .into_iter()
                                        .map(|provider| view! { <OAuth2ProviderButton provider=provider.clone() /> })
                                        .collect_view()
                                    }
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        }.into_any()
    }
}
