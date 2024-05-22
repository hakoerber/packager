use maud::{html, Markup, DOCTYPE};

pub(crate) struct ErrorPage;

impl ErrorPage {
    #[tracing::instrument]
    pub fn build(message: &str) -> Markup {
        html!(
            (DOCTYPE)
            html {
                head {
                    title { "Packager" }
                }
                body {
                    h1 { "Error" }
                    p { (message) }
                }
            }
        )
    }
}
