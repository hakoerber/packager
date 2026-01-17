use std::fmt::Display;

pub mod list;

pub enum HxSwap {
    OuterHtml,
}

pub enum Icon {
    Edit,
    Delete,
    Save,
    Cancel,
}

impl Icon {
    #[must_use]
    pub fn mdi_class(&self) -> &'static str {
        match self {
            Self::Edit => "mdi-pencil",
            Self::Delete => "mdi-delete",
            Self::Save => "mdi-content-save",
            Self::Cancel => "mdi-cancel",
        }
    }

    #[must_use]
    pub fn background(&self) -> &'static str {
        match self {
            Self::Edit => "bg-blue-100",
            Self::Delete => "bg-red-100",
            Self::Save => "bg-green-100",
            Self::Cancel => "bg-red-100",
        }
    }

    #[must_use]
    pub fn background_hover(&self) -> &'static str {
        match self {
            Self::Edit => "hover:bg-blue-400",
            Self::Delete => "hover:bg-red-400",
            Self::Save => "hover:bg-green-200",
            Self::Cancel => "hover:bg-red-200",
        }
    }
}

impl Display for HxSwap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::OuterHtml => "outerHtml",
            }
        )
    }
}

pub struct HxConfig {
    pub hx_post: String,
    pub hx_swap: HxSwap,
    pub hx_target: &'static str,
}
