use maud::{html, Markup};
use serde::{Deserialize, Serialize};

mod items;
mod model;
mod packagelist;
mod routes;
mod todos;
mod view;

pub(crate) use model::TripAttribute;
pub(crate) use routes::router;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AttributeValue<'a, T, I>(pub Option<&'a T>)
where
    T: std::fmt::Debug,
    Option<&'a T>: Input<Ids = I>;

impl<'a, T, I> AttributeValue<'a, T, I>
where
    T: std::fmt::Debug,
    Option<&'a T>: Input<Ids = I>,
{
    fn input(&self, id: I, form: &str) -> Markup {
        <Option<&'a T> as Input>::input(&self.0, id, form)
    }
}

impl<'a, T, I> From<&'a Option<T>> for AttributeValue<'a, T, I>
where
    T: std::fmt::Debug,
    Option<&'a T>: Input<Ids = I>,
{
    fn from(value: &'a Option<T>) -> Self {
        Self(value.as_ref())
    }
}

impl<'a, T, I> From<&'a T> for AttributeValue<'a, T, I>
where
    T: std::fmt::Debug,
    Option<&'a T>: Input<Ids = I>,
{
    fn from(value: &'a T) -> Self {
        Self(Some(value))
    }
}

pub(crate) trait Input {
    type Ids;

    fn input(&self, ids: Self::Ids, form: &str) -> Markup;
}

impl Input for Option<&model::Location> {
    type Ids = (&'static str,);
    fn input(&self, ids: Self::Ids, form: &str) -> Markup {
        html!(
            input ."m-auto" ."px-1" ."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
                type="number"
                id=(ids.0)
                name=(ids.0)
                form=(form)
                value=(self.map(|s| s.0.to_string()).unwrap_or_else(String::new))
            {}
        )
    }
}

impl Input for Option<&model::Name> {
    type Ids = (&'static str,);
    fn input(&self, ids: Self::Ids, form: &str) -> Markup {
        html!(
            input ."m-auto" ."px-1" ."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
                type="number"
                id=(ids.0)
                name=(ids.0)
                form=(form)
                value=(self.map(|s| s.0.to_string()).unwrap_or_else(String::new))
            {}
        )
    }
}

impl Input for Option<&model::Temperature> {
    type Ids = (&'static str,);
    fn input(&self, ids: Self::Ids, form: &str) -> Markup {
        html!(
            input ."m-auto" ."px-1" ."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
                type="number"
                id=(ids.0)
                name=(ids.0)
                form=(form)
                value=(self.map(|s| s.0.to_string()).unwrap_or_else(String::new))
            {}
        )
    }
}

impl Input for Option<&model::TripDate> {
    type Ids = (&'static str, &'static str);
    fn input(&self, ids: Self::Ids, form: &str) -> Markup {
        html!(
            input ."m-auto" ."px-1" ."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
                type="number"
                id=(ids.0)
                name=(ids.0)
                form=(form)
                value=(self.map(|s| s.to_string()).unwrap_or_else(String::new))
            {}
            input ."m-auto" ."px-1" ."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
                type="number"
                id=(ids.1)
                name=(ids.1)
                form=(form)
                value=(self.map(|s| s.to_string()).unwrap_or_else(String::new))
            {}

            p { "this could be your form!" }
        )
    }
}
