use leptos::prelude::*;

pub struct ErrorPage;

impl ErrorPage {
    #[tracing::instrument]
    pub fn build(message: &str) -> impl IntoView {
        todo!();
        view! {}
        // html!(
        //     (DOCTYPE)
        //     html {
        //         head {
        //             title { "Packager" }
        //         }
        //         body {
        //             h1 { "Error" }
        //             p { (message) }
        //         }
        //     }
        // )
    }
}
