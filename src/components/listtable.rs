use crate::elements::{
    list::{Action, Button},
    Icon,
};

use super::{
    types::{Name, Url},
    Render,
};

use bon::Builder;
use maud::{html, Markup};
use uuid::Uuid;

pub struct TextListWithDateRow<DelFn: Fn(Uuid) -> Url, EditFn: Fn(Uuid) -> Url> {
    id: Uuid,
    date: time::Date,
    content: String,
    delete: DelFn,
    edit: EditFn,
}

impl<DelFn: Fn(Uuid) -> Url, EditFn: Fn(Uuid) -> Url>
    From<(Uuid, time::Date, String, DelFn, EditFn)> for TextListWithDateRow<DelFn, EditFn>
{
    fn from((id, date, content, delete, edit): (Uuid, time::Date, String, DelFn, EditFn)) -> Self {
        Self {
            id,
            date,
            content,
            delete,
            edit,
        }
    }
}

impl<DelFn: Fn(Uuid) -> Url, EditFn: Fn(Uuid) -> Url> Render
    for TextListWithDateRow<DelFn, EditFn>
{
    fn render(&self) -> Markup {
        html!(
            tr ."h-10" ."even:bg-gray-100" ."hover:bg-gray-100" ."h-full" {
                td ."border" ."p-2" { ( self.date ) }
                td ."border" ."p-2" { ( self.content ) }
                (Button {
                    icon: Icon::Edit,
                    action: Action::Href((self.edit)(self.id)),
                    hx_config: None,
                }.render())
                (Button {
                    icon: Icon::Delete,
                    action: Action::Href((self.delete)(self.id)),
                    hx_config: None,
                }.render())
            }
        )
    }
}

#[derive(Builder)]
pub struct TextListWithDate<DelFn: Fn(Uuid) -> Url, EditFn: Fn(Uuid) -> Url> {
    ident: &'static str,
    name: Name,
    rows: Vec<TextListWithDateRow<DelFn, EditFn>>,
    new_row: Url,
}

impl<DelFn: Fn(Uuid) -> Url, EditFn: Fn(Uuid) -> Url> Render for TextListWithDate<DelFn, EditFn> {
    fn render(&self) -> Markup {
        let add_form_id = format!("new-{}", self.ident);

        html!(
            p ."text-xl" { (self.name.plural.capitalize().render()) }
            div
                ."flex"
                ."flex-col"
                ."gap-2"
            {
                table
                    ."table"
                    ."table-auto"
                    ."border-collapse"
                    ."border-spacing-0"
                    ."border"
                    ."w-full"
                {
                    tbody {
                        @for row in &self.rows {
                            (row.render())
                        }
                    }
                }
                div
                    x-data="{ save_active: false }"
                {
                    p { "add new " (self.name.singular.render()) }
                    form
                        name=(add_form_id)
                        id=(add_form_id)
                        action=(self.new_row.render())
                        target="_self"
                        method="post" {}

                    textarea
                        id="new-comment-content"
                        x-on:input=r#"save_active=(document.getElementById("new-comment-content").textLength) !== 0"#
                        ."border" ."w-full" ."h-24"
                        form=(add_form_id)
                        name="new-comment-content"
                        autocomplete="off"
                        oninput=r#"this.style.height = "";this.style.height = this.scrollHeight + 2 + "px""#
                    {}

                    button
                        type="submit"
                        form=(add_form_id)
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
                }
            }
        )
    }
}
