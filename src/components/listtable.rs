use maud::{html, Markup};

use super::{
    types::{Name, Url},
    Render,
};

pub(crate) trait Row: Render {}

macro_rules! impl_row_for_tuple {
    ( $(($i:literal: $ty:ident)),* ) => {
        impl<$($ty),*> Render for ($($ty),*,)
        where
            $( $ty: Render, )+
        {
            fn render(&self) -> Markup {
                #[allow(non_snake_case)]
                // https://stackoverflow.com/a/77932996
                let ( $( $ty, )+ ) = self;
                html!(
                    tr ."h-10" ."even:bg-gray-100" ."hover:bg-gray-100" ."h-full" {
                        $( td ."border" ."p-2" { ( $ty.render() ) } )+
                        td {
                            a
                                ."flex"
                                ."flex-row"
                                ."aspect-square"
                                ."bg-red-100"
                                ."hover:bg-red-200"
                                href=(format!("?delete_todo=foo"))
                                hx-post={
                                    "/foo"
                                }
                                hx-target="#todolist"
                                hx-swap="outerHTML"
                            {
                                span ."m-auto" ."mdi" ."mdi-delete-outline" ."text-xl" {}
                            }
                        }
                    }
                )
            }
        }

        impl<$($ty),*> Row for ($($ty),*,)
        where
            $( $ty: Render, )+
        {
        }
    }
}

crate::impl_for_all_tuples!(impl_row_for_tuple);

pub(crate) trait InfoRow: Render {}

macro_rules! impl_info_row_for_tuple {
    ( $(($i:literal: $ty:ident)),* ) => {
        impl<$($ty),*> Render for ($($ty),*,)
        where
            $( $ty: Render, )+
        {
            fn render(&self) -> Markup {
                #[allow(non_snake_case)]
                // https://stackoverflow.com/a/77932996
                let ( $( $ty, )+ ) = self;
                html!(
                    tr ."h-10" ."even:bg-gray-100" ."hover:bg-gray-100" ."h-full" {
                        $( td ."border" ."p-2" { ( $ty.render() ) } )+
                    }
                )
            }
        }

        impl<$($ty),*> InfoRow for ($($ty),*,)
        where
            $( $ty: Render, )+
        {
        }
    }
}

crate::impl_for_all_tuples!(impl_info_row_for_tuple);

pub(crate) struct InfoBox {
    rows: Vec<Box<dyn Row>>,
}

impl InfoBox {
    pub(crate) fn from_rows(rows: Vec<Box<dyn Row>>) -> Self {
        Self { rows }
    }
}

impl Render for InfoBox {
    fn render(&self) -> Markup {
        html!(
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
        )
    }
}

use bon::Builder;

#[derive(Builder)]
pub(crate) struct List<R: Row> {
    ident: &'static str,
    name: Name,
    rows: Vec<R>,
    new_row: Url,
}

impl<R: Row> Render for List<R> {
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
