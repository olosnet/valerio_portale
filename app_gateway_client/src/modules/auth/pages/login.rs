use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_navigate;

use crate::modules::base::toast_utils::{use_toast_ctx, toast_error};
use crate::stores::auth_store::use_auth;

#[component]
pub fn Login() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let toast = use_toast_ctx();

    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let toast = toast.clone();

    move || {
        if !auth.initial_check_done.get() {
            return view! {
                <div class="flex min-h-screen bg-secondary items-center justify-center">
                    <p class="text-muted-foreground">"Caricamento..."</p>
                </div>
            }.into_any();
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

                    match auth.login(&username.get_untracked(), &password.get_untracked()).await {
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

        view! {
            <Title text="Login - App Gateway"/>
            <div class="min-h-screen flex items-center justify-center bg-secondary">
                <div class="w-full max-w-sm mx-4">
                    <div class="bg-background rounded-lg border border-border shadow-sm p-6">
                        <div class="text-center mb-6">
                            <h1 class="text-2xl font-bold text-foreground">"App Gateway"</h1>
                            <p class="text-sm text-muted-foreground mt-1">"Accedi per continuare"</p>
                        </div>

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
                                    class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                                    placeholder="nome@esempio.it"
                                    disabled=move || loading.get()
                                />
                            </div>

                            <div>
                                <label for="password" class="block text-sm font-medium text-foreground mb-1">
                                    "Password"
                                </label>
                                <input
                                    id="password"
                                    type="password"
                                    prop:value=password
                                    on:input=move |e| password.set(event_target_value(&e))
                                    class="w-full px-3 py-2 rounded-md border border-border bg-background text-foreground text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                                    placeholder="••••••••"
                                    disabled=move || loading.get()
                                />
                            </div>

                            <button
                                on:click=on_submit
                                disabled=move || loading.get()
                                class="w-full px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 disabled:opacity-50 transition-opacity"
                            >
                                {move || if loading.get() { "Accesso in corso..." } else { "Accedi" }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }.into_any()
    }
}
