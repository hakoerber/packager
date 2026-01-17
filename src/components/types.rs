use maud::{html, Markup, PreEscaped};

use super::Render;

enum InnerString {
    Owned(String),
    Static(&'static str),
}

impl InnerString {
    fn to_inner_owned(&self) -> String {
        match self {
            Self::Owned(s) => s.clone(),
            Self::Static(s) => (*s).to_owned(),
        }
    }
}

pub struct NameString(InnerString);

impl From<&'static str> for NameString {
    fn from(value: &'static str) -> Self {
        Self(InnerString::Static(value))
    }
}

impl NameString {
    pub fn capitalize(&self) -> Self {
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

pub struct Name {
    pub singular: NameString,
    pub plural: NameString,
}

pub struct Url(pub String);

impl Render for Url {
    fn render(&self) -> Markup {
        PreEscaped(self.0.clone())
    }
}

pub struct Date(pub time::Date);

impl Render for Date {
    fn render(&self) -> Markup {
        PreEscaped(self.0.to_string())
    }
}

#[derive(Clone)]
pub struct Currency(pub crate::models::Currency);

impl Render for Currency {
    fn render(&self) -> Markup {
        PreEscaped(match self.0 {
            crate::models::Currency::Eur(amount) => format!("{amount}€"),
        })
    }
}

pub struct Link {
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

pub struct Empty;

impl Render for Empty {
    fn render(&self) -> Markup {
        PreEscaped(String::new())
    }
}

pub struct Raw(pub String);

impl Render for Raw {
    fn render(&self) -> Markup {
        PreEscaped(self.0.clone())
    }
}
