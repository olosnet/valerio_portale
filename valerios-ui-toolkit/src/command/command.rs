use leptos::prelude::*;

#[derive(Clone)]
pub struct CommandContext {
    pub query: RwSignal<String>,
}

impl CommandContext {
    pub fn new() -> Self {
        Self { query: RwSignal::new(String::new()) }
    }
}

pub fn use_command() -> CommandContext {
    expect_context::<CommandContext>()
}

#[component]
pub fn Command(
    children: ChildrenFn,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    provide_context(CommandContext::new());
    let extra = class.unwrap_or("");
    let cls = move || format!("flex h-full w-full flex-col overflow-hidden rounded-md bg-popover text-popover-foreground {}", extra);

    view! {
        <div data-slot="command" class=cls()>
            {children()}
        </div>
    }
}
