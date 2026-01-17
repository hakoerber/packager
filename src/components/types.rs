use maud::{html, Markup, PreEscaped};

use super::Render;

enum InnerString {
    Owned(String),
    Static(&'static str),
}

impl InnerString {
    fn to_inner_owned(&self) -> String {
        match self {
            InnerString::Owned(s) => s.clone(),
            InnerString::Static(s) => (*s).to_owned(),
        }
    }
}

pub(crate) struct NameString(InnerString);

impl From<&'static str> for NameString {
    fn from(value: &'static str) -> Self {
        Self(InnerString::Static(value))
    }
}

impl NameString {
    pub fn capitalize(&self) -> NameString {
        let value = self.0.to_inner_owned();
        let mut chars = value.chars();
        let first_char = chars.next().expect("string cannot be empty");
        let uppercase = first_char.to_uppercase().collect::<Vec<char>>();
        let new_string = uppercase.into_iter().chain(chars).collect::<String>();
        Self(InnerString::Owned(new_string))
    }
}

impl Render for NameString {
    fn render(&self) -> Markup {
        PreEscaped(self.0.to_inner_owned())
    }
}

pub(crate) struct Name {
    pub(crate) singular: NameString,
    pub(crate) plural: NameString,
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
