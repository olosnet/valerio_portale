use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <Title text="404 - Page not found"/>
        <div class="flex flex-col items-center justify-center min-h-[60vh] text-foreground">
            <h1 class="text-6xl font-bold text-primary">"404"</h1>
            <p class="mt-4 text-xl text-muted-foreground">"Pagina non trovata"</p>
            <a href="/" class="mt-6 text-primary underline hover:no-underline">
                "Torna alla dashboard"
            </a>
        </div>
    }
}
