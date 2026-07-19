#![allow(dead_code)]
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub theme: RwSignal<&'static str>,
    pub dark: RwSignal<bool>,
}

impl ThemeContext {
    pub fn toggle_dark(&self) {
        self.dark.update(|v| *v = !*v);
    }

    pub fn set(&self, name: &'static str) {
        self.theme.set(name);
    }

    pub fn set_dark(&self, value: bool) {
        self.dark.set(value);
    }
}

pub fn use_theme() -> ThemeContext {
    expect_context::<ThemeContext>()
}

#[component]
pub fn ThemeProvider(
    children: Children,
    #[prop(default = "default")] initial_theme: &'static str,
    #[prop(default = false)] default_dark: bool,
) -> impl IntoView {
    let theme = RwSignal::new(initial_theme);
    let dark = RwSignal::new(default_dark);

    provide_context(ThemeContext { theme, dark });

    view! {
        <div data-slot="theme-provider" class=move || {
            let t = theme.get();
            let d = if dark.get() { " dark" } else { "" };
            format!("theme-{}{}", t, d)
        }>
            {children()}
        </div>
    }
}
