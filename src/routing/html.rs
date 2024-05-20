use maud::{html, Markup};

#[must_use]
pub fn concat(a: &Markup, b: &Markup) -> Markup {
    html!((a)(b))
}
