use std::sync::Arc;

use leptos::prelude::*;

use crate::button::{Button, ButtonVariant};
use crate::dialog::*;
use crate::icon::Icon;

#[component]
pub fn ConfirmDeleteDialog(
    open: RwSignal<bool>,
    item_type: &'static str,
    on_confirm: Callback<()>,
) -> impl IntoView {
    view! {
        <Dialog open=open>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>"Elimina " {item_type}</DialogTitle>
                    <DialogDescription>
                        "Sei sicuro di voler eliminare questo " {item_type} "? Questa azione non può essere annullata."
                    </DialogDescription>
                </DialogHeader>
                <div class="flex items-center gap-3 rounded-md border border-destructive/20 bg-destructive/5 p-3 text-sm text-destructive">
                    {Icon::AlertCircle.render()}
                    <span>"L'elemento verrà eliminato permanentemente."</span>
                </div>
                <DialogFooter>
                    <Button
                        variant=ButtonVariant::Destructive
                        on_click=Arc::new(move || {
                            open.set(false);
                            on_confirm.run(());
                        })
                    >"Elimina"</Button>
                    <DialogClose>
                        <Button variant=ButtonVariant::Outline>"Annulla"</Button>
                    </DialogClose>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    }
}
