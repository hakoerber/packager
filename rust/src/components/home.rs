use maud::{html, Markup};

pub struct Home {
    doc: Markup,
}

impl Home {
    pub fn build() -> Self {
        let doc: Markup = html!(
            div id="home" class={"p-8" "max-w-xl"} {
                p {
                    a href="/inventory/" { "Inventory" }
                }
                p {
                    a href="/trips/" { "Trips" }
                }
            }
        );

        Self { doc }
    }
}

impl From<Home> for Markup {
    fn from(val: Home) -> Self {
        val.doc
    }
}
