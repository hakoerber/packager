use std::fmt;

use base64::Engine as _;
use sha2::{Digest, Sha256};

use crate::Context;
use maud::Markup;

pub mod error;
pub mod home;
pub mod inventory;
pub mod root;
pub mod trip;

pub use error::ErrorPage;
pub use root::Root;

#[derive(Debug)]
pub enum HtmxAction {
    Get(String),
}

impl fmt::Display for HtmxAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Get(path) => write!(f, "{path}"),
        }
    }
}

#[derive(Debug)]
pub enum FallbackAction {
    Get(String),
}

impl fmt::Display for FallbackAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Get(path) => write!(f, "{path}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentId(String);

impl ComponentId {
    #[tracing::instrument]
    // fn new() -> Self {
    // NOTE: this could also use a static AtomicUsize incrementing integer, which might be faster
    // Self(random::<u32>())
    // }
    #[tracing::instrument]
    fn html_id(&self) -> String {
        let id = {
            let mut hasher = Sha256::new();
            hasher.update(self.0.as_bytes());
            hasher.finalize()
        };

        // 9 bytes is enough to be unique
        // If this is divisible by 3, it means that we can base64-encode it without
        // any "=" padding
        //
        // cannot panic, as the output for sha256 will always be bit
        let id = &id[..9];

        // URL_SAFE because we cannot have slashes in the output
        base64::engine::general_purpose::URL_SAFE.encode(id)
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.html_id())
    }
}

#[derive(Debug)]
pub enum HtmxTarget {
    Myself,
    Component(ComponentId),
}

#[derive(Debug)]
pub struct HtmxComponent {
    id: ComponentId,
    action: HtmxAction,
    fallback_action: FallbackAction,
    target: HtmxTarget,
}

impl HtmxComponent {
    fn target(&self) -> &ComponentId {
        match self.target {
            HtmxTarget::Myself => &self.id,
            HtmxTarget::Component(ref id) => id,
        }
    }
}

#[derive(Debug)]
pub enum Parent {
    Root,
    Component(ComponentId),
}

impl From<Parent> for ComponentId {
    fn from(value: Parent) -> Self {
        match value {
            Parent::Root => ComponentId("/".into()),
            Parent::Component(c) => c,
        }
    }
}

impl From<ComponentId> for Parent {
    fn from(value: ComponentId) -> Self {
        Self::Component(value)
    }
}

pub trait Component {
    type Args;

    fn init(parent: Parent, args: Self::Args) -> Self;
    fn build(self, context: &Context) -> Markup;
}
