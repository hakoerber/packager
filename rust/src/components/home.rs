use maud::{html, Markup};

pub struct Home;

impl Home {
    pub fn build() -> Markup {
        html!(
            div id="home" class={"p-8" "max-w-xl"} {
                p {
                    a href="/inventory/" { "Inventory" }
                }
                p {
                    a href="/trips/" { "Trips" }
                }
            }
        )
    }
}
