use maud::{html, Markup, PreEscaped, DOCTYPE};

pub mod home;
pub mod inventory;
pub mod trip;

pub struct Root;

#[derive(PartialEq, Eq)]
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
                    meta name="htmx-config" content=r#"{"useTemplateFragments":true}"# {}
                }
                body
                {
                    header
                        #header
                        ."h-16"
                        ."bg-gray-200"
                        ."flex"
                        ."flex-row"
                        ."flex-nowrap"
                        ."justify-between"
                        ."items-stretch"
                    {
                        a
                            #home
                            hx-boost="true"
                            href="/"
                            ."flex"
                            ."flex-row"
                            ."items-center"
                            ."gap-3"
                            ."px-5"
                            ."hover:bg-gray-300"
                        {
                            img ."h-12" src="/assets/luggage.svg" {}
                            span
                                ."text-xl"
                                ."font-semibold"
                            { "Packager" }
                        }
                        nav
                            ."grow"
                            ."flex"
                            ."flex-row"
                            ."justify-center"
                            ."gap-x-10"
                            ."items-stretch"
                        {
                            a
                                href="/inventory/"
                                hx-boost="true"
                                #header-link-inventory
                                ."px-5"
                                ."flex"
                                ."h-full"
                                ."text-lg"
                                ."hover:bg-gray-300"

                                // invisible top border to fix alignment
                                ."border-t-gray-200"[active_page == &TopLevelPage::Inventory]
                                ."hover:border-t-gray-300"[active_page == &TopLevelPage::Inventory]

                                ."border-b-gray-500"[active_page == &TopLevelPage::Inventory]
                                ."border-y-4"[active_page == &TopLevelPage::Inventory]
                                ."font-bold"[active_page == &TopLevelPage::Inventory]
                            { span ."m-auto" ."font-semibold" { "Inventory" }}
                            a
                                href="/trips/"
                                hx-boost="true"
                                #header-link-trips
                                ."px-5"
                                ."flex"
                                ."h-full"
                                ."text-lg"
                                ."hover:bg-gray-300"

                                // invisible top border to fix alignment
                                ."border-t-gray-200"[active_page == &TopLevelPage::Trips]
                                ."hover:border-t-gray-300"[active_page == &TopLevelPage::Trips]

                                ."border-gray-500"[active_page == &TopLevelPage::Trips]
                                ."border-y-4"[active_page == &TopLevelPage::Trips]
                                ."font-bold"[active_page == &TopLevelPage::Trips]
                            { span ."m-auto" ."font-semibold" { "Trips" }}
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
