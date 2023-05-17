use maud::{html, Markup, DOCTYPE};

pub mod home;
pub mod inventory;
pub mod triplist;

mod theme;

pub use home::*;
pub use inventory::*;
pub use triplist::*;

pub struct Root {
    doc: Markup,
}

pub enum TopLevelPage {
    Inventory,
    Trips,
    None,
}

impl Root {
    pub fn build(body: Markup, active_page: &TopLevelPage) -> Self {
        let doc = html!(
            (DOCTYPE)
            html {
                head {
                    title { "Packager" }
                    script src="https://unpkg.com/htmx.org@1.7.0" {}
                    script src="https://cdn.tailwindcss.com" {}
                    script src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.js" defer {}
                    link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@mdi/font@6.9.96/css/materialdesignicons.min.css";
                    script { (include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/js/app.js"))) }
                }
                body {
                    header
                        ."bg-gray-200"
                        ."p-5"
                        ."flex"
                        ."flex-row"
                        ."flex-nowrap"
                        ."justify-between"
                        ."items-center"
                        hx-boost="true"
                    {
                        span ."text-xl" ."font-semibold" {
                            a href="/" { "Packager" }
                        }
                        nav ."grow" ."flex" ."flex-row" ."justify-center" ."gap-x-6" {
                            a href="/inventory/" class={@match active_page {
                                TopLevelPage::Inventory => "text-lg font-bold underline",
                                _ => "text-lg",
                            }} { "Inventory" }
                            a href="/trips/" class={@match active_page {
                                TopLevelPage::Trips => "text-lg font-bold underline",
                                _ => "text-lg",
                            }} { "Trips" }
                        }
                    }
                    div hx-boost="true" {
                        (body)
                    }
                }
            }
        );

        Self { doc }
    }

    pub fn into_string(self) -> String {
        self.doc.into_string()
    }
}
