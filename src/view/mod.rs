/*!
Each component is a struct that can be rendered

They form a tree, with a "build of" relationship

So each component can contain multiple other components
Each component has a parent, except the `root` element

Each component can send a single request (called action) which is
either done with or without Htmx.

Each component implements the [`Component`] trait. It has two
functions:

* `init()` to pass a reference to the parent and component-specific
  arguments (implemented as an associated type of the trait)
* `build()` that receives [`Context`] and emits Markup to render.

## Actions

Actions can either be done using Htmx or the standard way. To do this,
each component is supposed to contain a `HtmxComponent` struct. A `HtmxComponent`
contains the following information:

* `ComponentId` that uniquely (and stably) identifies the component (used for the
  HTML target ID)
* `action`: The action to take (HTTP method & URL)
* `fallback_action`: The action to take when Htmx is not available (HTTP method & URL)
* `target`: What to target for Htmx swap (either itself or a reference to another component)
*/

use std::fmt;

use base64::Engine as _;
use sha2::{Digest, Sha256};

use crate::Context;
use maud::Markup;

pub(crate) mod error;
pub(crate) mod home;
pub(crate) mod root;

pub(crate) use error::ErrorPage;
pub(crate) use root::Root;

#[derive(Debug)]
pub(crate) enum HtmxAction {
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
pub(crate) enum FallbackAction {
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
pub(crate) struct ComponentId(String);

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

    #[tracing::instrument]
    fn selector(&self) -> String {
        format!("#{}", self.html_id())
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.html_id())
    }
}

#[derive(Debug)]
pub(crate) enum HtmxTarget {
    Myself,
    Component(ComponentId),
}

#[derive(Debug)]
pub(crate) struct HtmxComponent {
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
pub(crate) enum Parent {
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

pub(crate) trait Component {
    type Args;

    fn init(parent: Parent, args: Self::Args) -> Self;
    fn build(self, context: &Context) -> Markup;
}
