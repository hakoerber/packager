use std::fmt::Display;

pub(crate) mod list;

pub(crate) enum HxSwap {
    OuterHtml,
}

pub(crate) enum Icon {
    Edit,
    Delete,
    Save,
    Cancel,
}

impl Icon {
    #[must_use]
    pub fn mdi_class(&self) -> &'static str {
        match self {
            Icon::Edit => "mdi-pencil",
            Icon::Delete => "mdi-delete",
            Icon::Save => "mdi-content-save",
            Icon::Cancel => "mdi-cancel",
        }
    }

    #[must_use]
    pub fn background(&self) -> &'static str {
        match self {
            Icon::Edit => "bg-blue-200",
            Icon::Delete => "bg-red-200",
            Icon::Save => "bg-green-100",
            Icon::Cancel => "bg-red-100",
        }
    }

    #[must_use]
    pub fn background_hover(&self) -> &'static str {
        match self {
            Icon::Edit => "hover:bg-blue-400",
            Icon::Delete => "hover:bg-red-400",
            Icon::Save => "hover:bg-green-200",
            Icon::Cancel => "hover:bg-red-200",
        }
    }
}

impl Display for HxSwap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HxSwap::OuterHtml => "outerHtml",
            }
        )
    }
}

pub(crate) struct HxConfig {
    pub hx_post: String,
    pub hx_swap: HxSwap,
    pub hx_target: &'static str,
}
