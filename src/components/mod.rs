pub mod infobox;
pub mod listtable;
pub mod text;
pub mod types;

pub use infobox::InfoBox;
pub use listtable::TextListWithDate;
pub use text::Text;

use maud::Markup;

#[allow(dead_code)]
pub trait Component: Render {}

pub trait Render {
    fn render(&self) -> Markup;
}

impl<T> Render for Option<T>
where
    T: Render,
{
    fn render(&self) -> Markup {
        self.as_ref().map_or_else(|| types::Empty.render(), Render::render)
    }
}
