use axohtml::{
    dom::DOMTree,
    elements::FlowContent,
    html,
    types::{Class, SpacedSet},
    unsafe_text,
};

type Tree = Box<dyn FlowContent<String>>;

pub mod home;
pub mod inventory;
pub mod triplist;

pub use home::*;
pub use inventory::*;
pub use triplist::*;

pub struct Root {
    doc: DOMTree<String>,
}

pub enum TopLevelPage {
    Inventory,
    Trips,
    None,
}

impl Root {
    pub fn build(body: Tree, active_page: TopLevelPage) -> Self {
        let active_classes: SpacedSet<Class> =
            ["text-lg", "font-bold", "underline"].try_into().unwrap();
        let inactive_classes: SpacedSet<Class> = ["text-lg"].try_into().unwrap();

        let doc = html!(
            <html>
                <head>
                    <title>"Packager"</title>
                    <script src="https://unpkg.com/htmx.org@1.7.0"/>
                    <script src="https://cdn.tailwindcss.com"/>
                    <script src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.js" defer="true"/>
                    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@mdi/font@6.9.96/css/materialdesignicons.min.css"/>
                    <script>{unsafe_text!(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/js/app.js")))}</script>
                </head>
                <body>
                    <header class=[
                        "bg-gray-200",
                        "p-5",
                        "flex",
                        "flex-row",
                        "flex-nowrap",
                        "justify-between",
                        "items-center",
                    ]>
                        <span class=["text-xl", "font-semibold"]>
                            <a href="/">"Packager"</a>
                        </span>
                        <nav class=["grow", "flex", "flex-row", "justify-center", "gap-x-6"]>
                            <a href="/inventory/" class={
                                match active_page {
                                    TopLevelPage::Inventory => active_classes.clone(),
                                    _ => inactive_classes.clone(),
                                }
                                }>"Inventory"</a>
                            <a href="/trips/" class={
                                match active_page {
                                    TopLevelPage::Trips => active_classes,
                                    _ => inactive_classes,
                                }
                                }>"Trips"</a>
                        </nav>
                    </header>
                    {body}
                </body>
            </html>
        );

        Self { doc }
    }

    pub fn to_string(&self) -> String {
        let mut doc = self.doc.to_string();
        doc.insert_str(0, "<!DOCTYPE html>\n");
        doc
    }
}
