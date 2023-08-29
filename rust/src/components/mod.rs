use maud::{html, Markup, PreEscaped, DOCTYPE};

pub mod home;
pub mod inventory;
pub mod trip;

pub use home::*;
pub use inventory::*;
pub use trip::*;

pub struct Root;

pub enum TopLevelPage {
    Inventory,
    Trips,
    None,
}

impl Root {
    pub fn build(body: &Markup, active_page: &TopLevelPage) -> Markup {
        html!(
            (DOCTYPE)
            html {
                head {
                    title { "Packager" }
                    script src="https://unpkg.com/htmx.org@1.9.2" {}
                    script src="https://unpkg.com/alpinejs@3.12.1" defer {}
                    script src="https://cdn.tailwindcss.com" {}
                    script src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.js" defer {}
                    link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@mdi/font@6.9.96/css/materialdesignicons.min.css" {}
                    link rel="shortcut icon" type="image/svg+xml" href="/favicon.svg" {}
                    script { (PreEscaped(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/js/app.js")))) }
                }
                body
                    hx-boost="true"
                {
                    header
                        #header
                        ."h-full"
                        ."bg-gray-200"
                        ."p-5"
                        ."flex"
                        ."flex-row"
                        ."flex-nowrap"
                        ."justify-between"
                        ."items-center"
                    {
                        span
                            ."text-xl"
                            ."font-semibold"
                            ."flex"
                            ."flex-row"
                            ."items-center"
                            ."gap-3"
                        {
                            img ."h-12" src="/assets/luggage.svg" {}
                            a #home href="/" { "Packager" }
                        }
                        nav
                            ."grow"
                            ."flex"
                            ."flex-row"
                            ."justify-center"
                            ."gap-x-10"
                            ."content-stretch"
                        {
                            a href="/inventory/"
                                #header-link-inventory
                                ."h-full"
                                ."text-lg"
                                ."font-bold"[matches!(active_page, TopLevelPage::Inventory)]
                                ."underline"[matches!(active_page, TopLevelPage::Inventory)]
                            { "Inventory" }
                            a href="/trips/"
                                #header-link-trips
                                ."h-full"
                                ."text-lg"
                                ."font-bold"[matches!(active_page, TopLevelPage::Trips)]
                                ."underline"[matches!(active_page, TopLevelPage::Trips)]
                            { "Trips" }
                        }
                    }
                    (body)
                }
            }
        )
    }
}

pub struct ErrorPage;

impl ErrorPage {
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
