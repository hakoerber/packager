use maud::{html, Markup};

pub fn concat(a: Markup, b: Markup) -> Markup {
    html!((a)(b))
}
