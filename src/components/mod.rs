pub(crate) mod infobox;
pub(crate) mod listtable;
pub(crate) mod text;
pub(crate) mod types;

pub(crate) use infobox::InfoBox;
pub(crate) use listtable::TextListWithDate;
pub(crate) use text::Text;

use maud::Markup;

#[allow(dead_code)]
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
