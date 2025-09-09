#![expect(dead_code)]

use maud::{html, Markup, PreEscaped};

pub(crate) trait Component {}

pub(crate) trait Render {
    fn render(&self) -> Markup;
}

pub(crate) struct Url(pub String);

impl Render for Url {
    fn render(&self) -> Markup {
        PreEscaped(self.0.clone())
    }
}

pub(crate) struct Date(pub time::Date);

impl Render for Date {
    fn render(&self) -> Markup {
        PreEscaped(self.0.to_string())
    }
}

#[derive(Clone)]
pub(crate) struct Currency(pub crate::models::Currency);

impl Render for Currency {
    fn render(&self) -> Markup {
        PreEscaped(match self.0 {
            crate::models::Currency::Eur(amount) => format!("{}€", amount),
        })
    }
}

pub(crate) struct Link {
    pub name: Option<String>,
    pub url: Url,
}

impl Render for Link {
    fn render(&self) -> Markup {
        html!(a
            ."text-blue-600"
            ."visited:text-purple-600"
            ."hover:underline"
            href=(self.url.0)
            {
                (self.name.clone().unwrap_or(self.url.0.clone()))
            }
        )
    }
}

pub(crate) struct Empty;

impl Render for Empty {
    fn render(&self) -> Markup {
        PreEscaped("".to_owned())
    }
}

pub(crate) struct Raw(pub String);

impl Render for Raw {
    fn render(&self) -> Markup {
        PreEscaped(self.0.clone())
    }
}

impl<T> Render for Option<T>
where
    T: Render,
{
    fn render(&self) -> Markup {
        self.as_ref()
            .map(|s| s.render())
            .unwrap_or_else(|| Empty.render())
    }
}

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

pub(crate) struct ListTable {
    rows: Vec<Box<dyn Row>>,
}

impl ListTable {
    pub(crate) fn from_rows(rows: Vec<Box<dyn Row>>) -> Self {
        Self { rows }
    }
}

impl Render for ListTable {
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
