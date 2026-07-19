pub mod sonner;
pub mod toast;
pub mod toast_stack;
pub mod types;

pub use sonner::*;
pub use toast::*;
pub use toast_stack::*;
pub use types::*;

#[macro_export]
macro_rules! toast {
    ($title:expr) => {
        $crate::use_toast().add.run(
            $crate::ToastInput {
                title: $title.to_string(),
                description: None,
                toast_type: $crate::ToastType::Default,
                action: None,
                duration_ms: 4000,
                callbacks: None,
            }
        );
    };
}

#[macro_export]
macro_rules! toast_success {
    ($title:expr) => {
        $crate::use_toast().add.run(
            $crate::ToastInput {
                title: $title.to_string(),
                description: None,
                toast_type: $crate::ToastType::Success,
                action: None,
                duration_ms: 4000,
                callbacks: None,
            }
        );
    };
}

#[macro_export]
macro_rules! toast_error {
    ($title:expr) => {
        $crate::use_toast().add.run(
            $crate::ToastInput {
                title: $title.to_string(),
                description: None,
                toast_type: $crate::ToastType::Error,
                action: None,
                duration_ms: 5000,
                callbacks: None,
            }
        );
    };
}

#[macro_export]
macro_rules! toast_warning {
    ($title:expr) => {
        $crate::use_toast().add.run(
            $crate::ToastInput {
                title: $title.to_string(),
                description: None,
                toast_type: $crate::ToastType::Warning,
                action: None,
                duration_ms: 4000,
                callbacks: None,
            }
        );
    };
}

#[macro_export]
macro_rules! toast_info {
    ($title:expr) => {
        $crate::use_toast().add.run(
            $crate::ToastInput {
                title: $title.to_string(),
                description: None,
                toast_type: $crate::ToastType::Info,
                action: None,
                duration_ms: 4000,
                callbacks: None,
            }
        );
    };
}
