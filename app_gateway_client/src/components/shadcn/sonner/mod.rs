pub mod sonner;
pub mod toast;
pub mod toast_stack;
pub mod types;

#[allow(unused_imports)]
pub use sonner::*;
#[allow(unused_imports)]
pub use toast::*;
#[allow(unused_imports)]
pub use toast_stack::*;
#[allow(unused_imports)]
pub use types::*;

#[macro_export]
macro_rules! toast {
    ($title:expr) => {
        $crate::components::shadcn::sonner::use_toast().add.run(
            $crate::components::shadcn::sonner::ToastInput {
                title: $title.to_string(),
                description: None,
                toast_type: $crate::components::shadcn::sonner::ToastType::Default,
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
        $crate::components::shadcn::sonner::use_toast().add.run(
            $crate::components::shadcn::sonner::ToastInput {
                title: $title.to_string(),
                description: None,
                toast_type: $crate::components::shadcn::sonner::ToastType::Success,
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
        $crate::components::shadcn::sonner::use_toast().add.run(
            $crate::components::shadcn::sonner::ToastInput {
                title: $title.to_string(),
                description: None,
                toast_type: $crate::components::shadcn::sonner::ToastType::Error,
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
        $crate::components::shadcn::sonner::use_toast().add.run(
            $crate::components::shadcn::sonner::ToastInput {
                title: $title.to_string(),
                description: None,
                toast_type: $crate::components::shadcn::sonner::ToastType::Warning,
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
        $crate::components::shadcn::sonner::use_toast().add.run(
            $crate::components::shadcn::sonner::ToastInput {
                title: $title.to_string(),
                description: None,
                toast_type: $crate::components::shadcn::sonner::ToastType::Info,
                action: None,
                duration_ms: 4000,
                callbacks: None,
            }
        );
    };
}
