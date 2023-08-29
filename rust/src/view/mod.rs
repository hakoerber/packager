use super::Context;

use maud::{html, Markup, PreEscaped, DOCTYPE};

pub mod home;
pub mod inventory;
pub mod trip;

pub struct Root;

use crate::TopLevelPage;

impl Root {
    pub fn build(context: &Context, body: &Markup, active_page: Option<&TopLevelPage>) -> Markup {
        let menu_item = |item: TopLevelPage, active_page: Option<&TopLevelPage>| {
            let active = active_page.map_or(false, |page| *page == item);
            html!(
                a
                    href=(item.path())
                    #{"header-link-" (item.id())}
                    ."px-5"
                    ."flex"
                    ."h-full"
                    ."text-lg"
                    ."hover:bg-gray-300"

                    // invisible top border to fix alignment
                    ."border-t-gray-200"[active]
                    ."hover:border-t-gray-300"[active]

                    ."border-b-gray-500"[active]
                    ."border-y-4"[active]
                    ."font-bold"[active]
                { span ."m-auto" ."font-semibold" { (item.name()) }}
            )
        };

        html!(
            (DOCTYPE)
            html {
                head {
                    title { "Packager" }
                    script src="https://unpkg.com/htmx.org@1.9.4" {}
                    script src="https://unpkg.com/alpinejs@3.12.3" defer {}
                    script src="https://cdn.tailwindcss.com/3.3.3" {}
                    link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@mdi/font@7.2.96/css/materialdesignicons.min.css" {}
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
                            (menu_item(TopLevelPage::Inventory, active_page))
                            (menu_item(TopLevelPage::Trips, active_page))
                        }
                        a
                            ."flex"
                            ."flex-row"
                            ."items-center"
                            ."gap-3"
                            ."px-5"
                            ."bg-gray-200"
                            ."hover:bg-gray-300"
                            href=(format!("/user/{}", context.user.id))
                        {
                            span
                                ."m-auto"
                                ."mdi"
                                ."mdi-account"
                                ."text-3xl"
                            {}
                            p { (context.user.fullname)}
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
