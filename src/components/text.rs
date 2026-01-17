use bon::Builder;
use maud::{html, Markup};
use uuid::Uuid;

use super::{types::Url, Render};

#[derive(Builder)]
pub(crate) struct Text<SaveFn: Fn(Uuid) -> Url, CancelFn: Fn(Uuid) -> Url> {
    id: Uuid,
    initial_content: String,
    save: SaveFn,
    cancel: CancelFn,
}

impl<SaveFn: Fn(Uuid) -> Url, CancelFn: Fn(Uuid) -> Url> Render for Text<SaveFn, CancelFn> {
    fn render(&self) -> Markup {
        let id: &'static str = "text";
        html!(
            form
                name=(id)
                id=(id)
                action=((self.save)(self.id).render())
                target="_self"
                method="post" {}

            textarea
                id="new-content"
                x-on:input=r#"save_active=(document.getElementById("new-content").textLength) !== 0"#
                ."border" ."w-full" ."h-24"
                form=(id)
                name="new-content"
                autocomplete="off"
                oninput=r#"this.style.height = "";this.style.height = this.scrollHeight + 2 + "px""#
            {
                (self.initial_content)
            }

            div
                ."flex"
                ."flex-row"
                ."gap-2"
            {
                button
                    type="submit"
                    form=(id)
                    x-bind:disabled="!save_active"
                    ."enabled:bg-green-200"
                    ."enabled:hover:bg-green-400"
                    ."enabled:cursor-pointer"
                    ."disabled:opacity-50"
                    ."disabled:bg-gray-300"
                    ."mt-2"
                    ."border"
                    ."flex"
                    ."flex-column"
                    ."p-2"
                    ."gap-2"
                    ."items-center"
                {
                    span
                        ."mdi"
                        ."mdi-content-save"
                        ."text-xl" {}
                    span { "Save" }
                }
                a
                    ."bg-gray-200"
                    ."hover:bg-gray-400"
                    ."mt-2"
                    ."border"
                    ."flex"
                    ."flex-column"
                    ."cursor-pointer"
                    ."p-2"
                    ."gap-2"
                    ."items-center"
                    href=((self.cancel)(self.id).render())
                {
                    span
                        ."mdi"
                        ."mdi-cancel"
                        ."text-xl" {}
                    span { "Cancel" }
                }
            }
        )
    }
}
