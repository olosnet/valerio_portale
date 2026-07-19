#![allow(dead_code)]
use leptos::prelude::*;

macro_rules! icon_svg {
    ($($inner:tt)*) => {
        view! {
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"
                 viewBox="0 0 24 24" fill="none" stroke="currentColor"
                 stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                 class="size-4 shrink-0">
                $($inner)*
            </svg>
        }.into_any()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Icon {
    LayoutDashboard,
    User,
    Users,
    Shield,
    Settings,
    SlidersHorizontal,
    LogOut,
    ChevronRight,
    ChevronDown,
    ChevronLeft,
    ChevronUp,
    ChevronsUpDown,
    Menu,
    PanelLeft,
    PanelLeftOpen,
    ExternalLink,
    Plus,
    Pencil,
    Trash,
    Check,
    X,
    Search,
    Filter,
    ArrowLeft,
    ArrowRight,
    AlertCircle,
    AlertTriangle,
    CheckCircle,
    Info,
    Loader,
    Sun,
    Moon,
    FileText,
    Folder,
    MoreHorizontal,
    Copy,
    Eye,
    EyeOff,
    Mail,
    Home,
    Star,
    Upload,
    Download,
    Minus,
    PlusCircle,
    Ellipsis,
}

impl Icon {
    pub fn render(&self) -> AnyView {
        match self {
            Self::LayoutDashboard => icon_layout_dashboard(),
            Self::User => icon_user(),
            Self::Users => icon_users(),
            Self::Shield => icon_shield(),
            Self::Settings => icon_settings(),
            Self::SlidersHorizontal => icon_sliders_horizontal(),
            Self::LogOut => icon_log_out(),
            Self::ChevronRight => icon_chevron_right(),
            Self::ChevronDown => icon_chevron_down(),
            Self::ChevronLeft => icon_chevron_left(),
            Self::ChevronUp => icon_chevron_up(),
            Self::ChevronsUpDown => icon_chevrons_up_down(),
            Self::Menu => icon_menu(),
            Self::PanelLeft => icon_panel_left(),
            Self::PanelLeftOpen => icon_panel_left_open(),
            Self::ExternalLink => icon_external_link(),
            Self::Plus => icon_plus(),
            Self::Pencil => icon_pencil(),
            Self::Trash => icon_trash(),
            Self::Check => icon_check(),
            Self::X => icon_x(),
            Self::Search => icon_search(),
            Self::Filter => icon_filter(),
            Self::ArrowLeft => icon_arrow_left(),
            Self::ArrowRight => icon_arrow_right(),
            Self::AlertCircle => icon_alert_circle(),
            Self::AlertTriangle => icon_alert_triangle(),
            Self::CheckCircle => icon_check_circle(),
            Self::Info => icon_info(),
            Self::Loader => icon_loader(),
            Self::Sun => icon_sun(),
            Self::Moon => icon_moon(),
            Self::FileText => icon_file_text(),
            Self::Folder => icon_folder(),
            Self::MoreHorizontal => icon_more_horizontal(),
            Self::Copy => icon_copy(),
            Self::Eye => icon_eye(),
            Self::EyeOff => icon_eye_off(),
            Self::Mail => icon_mail(),
            Self::Home => icon_home(),
            Self::Star => icon_star(),
            Self::Upload => icon_upload(),
            Self::Download => icon_download(),
            Self::Minus => icon_minus(),
            Self::PlusCircle => icon_plus_circle(),
            Self::Ellipsis => icon_ellipsis(),
        }
    }
}

// ------------------- NAVIGAZIONE -------------------

fn icon_layout_dashboard() -> AnyView {
    icon_svg!(
        <rect width="7" height="9" x="3" y="3" rx="1"/>
        <rect width="7" height="5" x="14" y="3" rx="1"/>
        <rect width="7" height="9" x="14" y="12" rx="1"/>
        <rect width="7" height="5" x="3" y="16" rx="1"/>
    )
}

fn icon_user() -> AnyView {
    icon_svg!(
        <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/>
        <circle cx="12" cy="7" r="4"/>
    )
}

fn icon_users() -> AnyView {
    icon_svg!(
        <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/>
        <circle cx="9" cy="7" r="4"/>
        <path d="M22 21v-2a4 4 0 0 0-3-3.87"/>
        <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
    )
}

fn icon_shield() -> AnyView {
    icon_svg!(
        <path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/>
    )
}

fn icon_settings() -> AnyView {
    icon_svg!(
        <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
        <circle cx="12" cy="12" r="3"/>
    )
}

fn icon_sliders_horizontal() -> AnyView {
    icon_svg!(
        <path d="M14 4h6"/><path d="M4 12h16"/><path d="M4 20h6"/>
        <circle cx="17" cy="17" r="3"/><circle cx="7" cy="7" r="3"/>
    )
}

fn icon_log_out() -> AnyView {
    icon_svg!(
        <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
        <polyline points="16 17 21 12 16 7"/>
        <line x1="21" x2="9" y1="12" y2="12"/>
    )
}

fn icon_chevron_right() -> AnyView {
    icon_svg!(<path d="m9 18 6-6-6-6"/>)
}

fn icon_chevron_down() -> AnyView {
    icon_svg!(<path d="m6 9 6 6 6-6"/>)
}

fn icon_chevron_left() -> AnyView {
    icon_svg!(<path d="m15 18-6-6 6-6"/>)
}

fn icon_chevron_up() -> AnyView {
    icon_svg!(<path d="m18 15-6-6-6 6"/>)
}

fn icon_chevrons_up_down() -> AnyView {
    icon_svg!(
        <path d="m7 15 5 5 5-5"/><path d="m7 9 5-5 5 5"/>
    )
}

fn icon_menu() -> AnyView {
    icon_svg!(
        <line x1="4" x2="20" y1="12" y2="12"/>
        <line x1="4" x2="20" y1="6" y2="6"/>
        <line x1="4" x2="20" y1="18" y2="18"/>
    )
}

fn icon_panel_left() -> AnyView {
    icon_svg!(
        <rect width="18" height="18" x="3" y="3" rx="2"/>
        <path d="M9 3v18"/>
    )
}

fn icon_panel_left_open() -> AnyView {
    icon_svg!(
        <rect width="18" height="18" x="3" y="3" rx="2"/>
        <path d="M9 3v18"/>
        <path d="m14 9 3 3-3 3"/>
    )
}

fn icon_external_link() -> AnyView {
    icon_svg!(
        <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
        <polyline points="15 3 21 3 21 9"/>
        <line x1="10" y1="14" x2="21" y2="3"/>
    )
}

// ------------------- AZIONI -------------------

fn icon_plus() -> AnyView {
    icon_svg!(<path d="M5 12h14"/><path d="M12 5v14"/>)
}

fn icon_pencil() -> AnyView {
    icon_svg!(
        <path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/>
        <path d="m15 5 4 4"/>
    )
}

fn icon_trash() -> AnyView {
    icon_svg!(
        <path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
        <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
        <line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/>
    )
}

fn icon_check() -> AnyView {
    icon_svg!(<path d="M20 6 9 17l-5-5"/>)
}

fn icon_x() -> AnyView {
    icon_svg!(<path d="M18 6 6 18"/><path d="m6 6 12 12"/>)
}

fn icon_search() -> AnyView {
    icon_svg!(<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>)
}

fn icon_filter() -> AnyView {
    icon_svg!(<path d="M20 7h-9"/><path d="M14 17H5"/><circle cx="17" cy="17" r="3"/><circle cx="7" cy="7" r="3"/>)
}

fn icon_arrow_left() -> AnyView {
    icon_svg!(<path d="m12 19-7-7 7-7"/><path d="M19 12H5"/>)
}

fn icon_arrow_right() -> AnyView {
    icon_svg!(<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>)
}

// ------------------- STATO -------------------

fn icon_alert_circle() -> AnyView {
    icon_svg!(
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
    )
}

fn icon_alert_triangle() -> AnyView {
    icon_svg!(
        <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/>
        <line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
    )
}

fn icon_check_circle() -> AnyView {
    icon_svg!(<circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/>)
}

fn icon_info() -> AnyView {
    icon_svg!(
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="16" x2="12" y2="12"/>
        <line x1="12" y1="8" x2="12.01" y2="8"/>
    )
}

fn icon_loader() -> AnyView {
    icon_svg!(<path d="M21 12a9 9 0 1 1-6.219-8.56"/>)
}

// ------------------- LAYOUT / MISC -------------------

fn icon_sun() -> AnyView {
    icon_svg!(
        <circle cx="12" cy="12" r="4"/>
        <path d="M12 2v2"/><path d="M12 20v2"/>
        <path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/>
        <path d="M2 12h2"/><path d="M20 12h2"/>
        <path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>
    )
}

fn icon_moon() -> AnyView {
    icon_svg!(<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>)
}

fn icon_file_text() -> AnyView {
    icon_svg!(
        <path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/>
        <path d="M14 2v4a2 2 0 0 0 2 2h4"/>
        <path d="M10 9H8"/><path d="M16 13H8"/><path d="M16 17H8"/>
    )
}

fn icon_folder() -> AnyView {
    icon_svg!(
        <path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>
    )
}

fn icon_more_horizontal() -> AnyView {
    icon_svg!(
        <circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>
    )
}

fn icon_copy() -> AnyView {
    icon_svg!(
        <rect width="14" height="14" x="8" y="8" rx="2" ry="2"/>
        <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>
    )
}

fn icon_eye() -> AnyView {
    icon_svg!(
        <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
        <circle cx="12" cy="12" r="3"/>
    )
}

fn icon_eye_off() -> AnyView {
    icon_svg!(
        <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
        <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24"/>
        <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/>
        <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/>
        <line x1="2" x2="22" y1="2" y2="22"/>
    )
}

fn icon_mail() -> AnyView {
    icon_svg!(
        <rect width="20" height="16" x="2" y="4" rx="2"/>
        <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>
    )
}

fn icon_home() -> AnyView {
    icon_svg!(
        <path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/>
        <path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
    )
}

fn icon_star() -> AnyView {
    icon_svg!(
        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
    )
}

fn icon_upload() -> AnyView {
    icon_svg!(
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
        <polyline points="17 8 12 3 7 8"/>
        <line x1="12" x2="12" y1="3" y2="15"/>
    )
}

fn icon_download() -> AnyView {
    icon_svg!(
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
        <polyline points="7 10 12 15 17 10"/>
        <line x1="12" x2="12" y1="15" y2="3"/>
    )
}

fn icon_minus() -> AnyView {
    icon_svg!(<path d="M5 12h14"/>)
}

fn icon_plus_circle() -> AnyView {
    icon_svg!(
        <circle cx="12" cy="12" r="10"/>
        <path d="M8 12h8"/><path d="M12 8v8"/>
    )
}

fn icon_ellipsis() -> AnyView {
    icon_svg!(
        <circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>
    )
}
