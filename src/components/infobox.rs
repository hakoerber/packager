use maud::{html, Markup};

use super::Render;

pub trait InfoRow: Render {}

macro_rules! impl_for_all_tuples {
    ($name: ident) => {
        $name!((0: T0));
        $name!((0: T0), (1:T1));
        $name!((0: T0), (1:T1), (2:T2));
    }
}

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

impl_for_all_tuples!(impl_info_row_for_tuple);

pub struct InfoBox {
    rows: Vec<Box<dyn InfoRow>>,
}

impl InfoBox {
    pub fn from_rows(rows: Vec<Box<dyn InfoRow>>) -> Self {
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
