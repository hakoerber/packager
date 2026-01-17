pub(crate) mod listtable;
pub(crate) mod types;

pub(crate) use listtable::{InfoBox, List};

use maud::Markup;

#[macro_export]
macro_rules! impl_for_all_tuples {
    ($name: ident) => {
        $name!((0: T0));
        $name!((0: T0), (1:T1));
        $name!((0: T0), (1:T1), (2:T2));
    }
}

pub(crate) trait Component: Render {}

pub(crate) trait Render {
    fn render(&self) -> Markup;
}

impl<T> Render for Option<T>
where
    T: Render,
{
    fn render(&self) -> Markup {
        self.as_ref()
            .map(|s| s.render())
            .unwrap_or_else(|| types::Empty.render())
    }
}
