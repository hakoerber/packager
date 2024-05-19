use maud::{html, Markup};

use super::HxConfig;

pub struct Link<'a> {
    pub text: &'a str,
    pub href: String,
    pub hx_config: Option<HxConfig>,
}

pub struct NumberWithBar {
    pub value: i32,
    pub max_value: i32,
}

pub enum CellType<'a> {
    Text(&'a str),
    Link(Link<'a>),
    NumberWithBar(NumberWithBar),
}

pub struct Cell<'a> {
    pub cell_type: CellType<'a>,
}

impl<'a> Cell<'a> {
    fn render(self, _is_edit: bool) -> Markup {
        match self.cell_type {
            CellType::Text(text) => html!(
                td
                    ."border"
                    ."p-0"
                    ."m-0"
                {
                    p { (text) }
                }
            ),
            CellType::Link(link) => {
                let (hx_post, hx_swap, hx_target) = if let Some(hx_config) = link.hx_config {
                    (
                        Some(hx_config.hx_post),
                        Some(hx_config.hx_swap),
                        Some(hx_config.hx_target),
                    )
                } else {
                    (None, None, None)
                };
                html!(
                    td
                        ."border"
                        ."p-0"
                        ."m-0"
                    {
                        a
                            ."inline-block"
                            ."p-2"
                            ."m-0"
                            ."w-full"

                            href=(link.href)
                            hx-post=[hx_post]
                            hx-swap=[hx_swap]
                            hx-target=[hx_target]
                        {
                            (link.text)
                        }
                    }
                )
            }
            CellType::NumberWithBar(number) => html!(
                td
                    ."border"
                    ."p-2"
                    ."m-0"
                     style="position:relative;"
                {
                    p {
                        (number.value)
                    }
                    div ."bg-blue-600" ."h-1.5"
                        style=(
                            format!(
                                "width: {width}%;position:absolute;left:0;bottom:0;right:0;",
                                width=(
                                    (number.value as f64)
                                    / (number.max_value as f64)
                                    * 100.0
                                )
                            )
                        )
                    {}
                }
            ),
        }
    }
}

pub struct Button {
    pub icon: super::Icon,
    pub action: Action,
    pub hx_config: Option<HxConfig>,
}

impl Button {
    fn render(self) -> Markup {
        html!(
            td
                ."border-none"
                ."p-0"
                .(self.icon.background())
                .(self.icon.background_hover())
                ."h-full"
                ."w-10"
                {
                    @match self.action {
                        Action::Href(href) => {
                            a
                                href=(href)
                                ."aspect-square"
                                ."flex"
                            {
                                span
                                    ."m-auto"
                                    ."mdi"
                                    ."text-xl"
                                    .(self.icon.mdi_class())
                                {}
                            }
                        }
                        Action::Submit(form) => {
                            button
                                ."aspect-square"
                                ."flex"
                                ."w-full"
                                ."h-full"
                                type="submit"
                                form=(form)
                            {
                                span
                                    ."m-auto"
                                    ."mdi"
                                    .(self.icon.mdi_class())
                                    ."text-xl"
                                {}
                            }
                        }
                    }
            }
        )
    }
}

pub enum Action {
    Href(String),
    Submit(&'static str),
}

pub struct EditingConfig {
    pub edit_action: Action,
    pub edit_hx_config: Option<HxConfig>,
    pub delete_action: Action,
    pub delete_hx_config: Option<HxConfig>,
    pub save_action: Action,
    pub save_hx_config: Option<HxConfig>,
    pub cancel_action: Action,
    pub cancel_hx_config: Option<HxConfig>,
}

pub trait Row {
    fn is_active(&self) -> bool {
        false
    }

    fn is_edit(&self) -> bool {
        false
    }

    fn cells(&self) -> Vec<Cell>;
}

pub struct Header<'c> {
    pub cells: Vec<Option<HeaderCell<'c>>>,
}

pub struct HeaderCell<'a> {
    pub title: &'a str,
}

impl<'c> HeaderCell<'c> {
    fn title(&self) -> &str {
        &self.title
    }
}

pub struct List<'hc, R>
where
    R: Row,
{
    pub id: Option<&'static str>,
    pub header: Header<'hc>,
    pub rows: Vec<R>,
    pub editing_config: Option<Box<dyn Fn(R) -> EditingConfig>>,
}

impl<'hc, R> List<'hc, R>
where
    R: Row,
{
    pub fn render(self) -> Markup {
        html!(
            table
                id=[self.id]
                ."table"
                ."table-auto"
                ."border-collapse"
                ."border-spacing-0"
                ."border"
                ."w-full"
            {
                thead ."bg-gray-200" {
                    tr
                        ."h-10"
                    {
                        @for header_cell in self.header.cells.iter() {
                            th ."border" ."p-2" { (header_cell.as_ref().map_or("", |c| c.title())) }
                        }
                        @if self.editing_config.is_some() {
                            th {}
                            th {}
                        }
                    }
                }
                tbody {
                    @for row in self.rows.into_iter() {
                        @let active = row.is_active();
                        @let is_edit = row.is_edit();
                        tr
                            ."h-10"
                            ."hover:bg-gray-100"
                            ."outline"[active]
                            ."outline-2"[active]
                            ."outline-indigo-300"[active]
                            ."pointer-events-none"[active]
                            ."font-bold"[active]
                        {
                            @for cell in row.cells() {
                                (cell.render(is_edit))
                            }
                            @if let Some(ref edit_config) = self.editing_config {
                                @let edit_config = (*edit_config)(row);
                                @if is_edit {
                                    (Button {
                                        icon: super::Icon::Save,
                                        action: edit_config.save_action,
                                        hx_config: edit_config.save_hx_config,
                                    }.render())
                                    (Button {
                                        icon: super::Icon::Cancel,
                                        action: edit_config.cancel_action,
                                        hx_config: edit_config.cancel_hx_config,
                                    }.render())
                                } @else {
                                    (Button {
                                        icon: super::Icon::Edit,
                                        action: edit_config.edit_action,
                                        hx_config: edit_config.edit_hx_config,
                                    } .render())
                                    (Button {
                                        icon: super::Icon::Delete,
                                        action: edit_config.delete_action,
                                        hx_config: edit_config.delete_hx_config,
                                    }.render())
                                }

                            }
                        }
                    }
                }
            }
        )
    }
}
