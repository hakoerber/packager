use crate::{Context, TopLevelPage};

use maud::{html, Markup, PreEscaped, DOCTYPE};

use super::{
    Component, ComponentId, FallbackAction, HtmxAction, HtmxComponent, HtmxTarget, Parent,
};

pub struct Header;

impl Header {
    #[tracing::instrument]
    pub fn build() -> Markup {
        html!(
            head {
                title { "Packager" }
                script src="https://unpkg.com/htmx.org@1.9.4" {}
                script src="https://unpkg.com/alpinejs@3.12.3" defer {}
                script src="https://cdn.tailwindcss.com/3.3.3" {}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@mdi/font@7.2.96/css/materialdesignicons.min.css" {}
                link rel="shortcut icon" type="image/svg+xml" href="/favicon.svg" {}
                script { (PreEscaped(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/js/app.js")))) }
                meta name="htmx-config" content=r#"{"useTemplateFragments":true}"# {}
            }
        )
    }
}

pub struct HeaderLink<'a> {
    htmx: HtmxComponent,
    args: HeaderLinkArgs<'a>,
}

pub struct HeaderLinkArgs<'a> {
    pub item: TopLevelPage,
    pub active_page: Option<&'a TopLevelPage>,
}

impl<'a> Component for HeaderLink<'a> {
    type Args = HeaderLinkArgs<'a>;

    #[tracing::instrument(skip(args))]
    fn init(parent: Parent, args: Self::Args) -> Self {
        Self {
            htmx: HtmxComponent {
                id: ComponentId(format!("/header/component/{}", args.item.id())),
                action: HtmxAction::Get(args.item.path().to_string()),
                fallback_action: FallbackAction::Get(args.item.path().to_string()),
                target: HtmxTarget::Component(parent.into()),
            },
            args,
        }
    }

    #[tracing::instrument(skip(self))]
    fn build(self, context: &Context) -> Markup {
        let active = self
            .args
            .active_page
            .map_or(false, |page| *page == self.args.item);
        html!(
            a
                href=(self.args.item.path())
                hx-get=(self.args.item.path())
                hx-target={ "#" (self.htmx.target().html_id()) }
                hx-swap="outerHtml"
                #{"header-link-" (self.args.item.id())}
                ."px-5"
                ."flex"
                ."h-full"
                ."text-lg"
                ."hover:bg-gray-300"

                // invisible top border to fix alignment
                ."border-t-gray-200"[active]
                ."hover:border-t-gray-300"[active]

                ."border-b-gray-500"[active]
                ."border-y-4"[active]
                ."font-bold"[active]
            { span ."m-auto" ."font-semibold" { (self.args.item.name()) }}
        )
    }
}

pub struct Body<'a> {
    htmx: HtmxComponent,
    args: BodyArgs<'a>,
}

pub struct BodyArgs<'a> {
    pub body: &'a Markup,
    pub active_page: Option<&'a TopLevelPage>,
}

impl<'a> Component for Body<'a> {
    type Args = BodyArgs<'a>;

    #[tracing::instrument(skip(args))]
    fn init(parent: Parent, args: Self::Args) -> Self {
        Self {
            htmx: HtmxComponent {
                id: ComponentId("/body/".into()),
                action: HtmxAction::Get("/".into()),
                fallback_action: FallbackAction::Get("/".into()),
                target: HtmxTarget::Myself,
            },
            args,
        }
    }

    #[tracing::instrument(skip(self))]
    fn build(self, context: &Context) -> Markup {
        html!(
            body #(self.htmx.id.html_id())
            {
                header
                    #header
                    ."h-16"
                    ."bg-gray-200"
                    ."flex"
                    ."flex-row"
                    ."flex-nowrap"
                    ."justify-between"
                    ."items-stretch"
                {
                    a
                        #home
                        href=(self.htmx.fallback_action)
                        hx-get=(self.htmx.action)
                        hx-target={ "#" (self.htmx.target()) }
                        hx-swap="outerHTML"
                        ."flex"
                        ."flex-row"
                        ."items-center"
                        ."gap-3"
                        ."px-5"
                        ."hover:bg-gray-300"
                    {
                        img ."h-12" src="/assets/luggage.svg" {}
                        span
                            ."text-xl"
                            ."font-semibold"
                        { "Packager" }
                    }
                    nav
                        ."grow"
                        ."flex"
                        ."flex-row"
                        ."justify-center"
                        ."gap-x-10"
                        ."items-stretch"
                    {
                        (
                            // todo make clone() unnecessary
                            // make ComponentId take &str instead of owned string
                            HeaderLink::init(
                                self.htmx.id.clone().into(),
                                HeaderLinkArgs {
                                    item: TopLevelPage::Inventory,
                                    active_page: self.args.active_page
                                }
                            ).build(&context)
                        )
                        (
                            HeaderLink::init(
                                self.htmx.id.clone().into(),
                                HeaderLinkArgs {
                                    item: TopLevelPage::Trips,
                                    active_page: self.args.active_page
                                }
                            ).build(&context)
                        )
                    }
                    a
                        ."flex"
                        ."flex-row"
                        ."items-center"
                        ."gap-3"
                        ."px-5"
                        ."bg-gray-200"
                        ."hover:bg-gray-300"
                        href=(format!("/user/{}", context.user.id))
                    {
                        span
                            ."m-auto"
                            ."mdi"
                            ."mdi-account"
                            ."text-3xl"
                        {}
                        p { (context.user.fullname)}
                    }
                }
                (self.args.body)
            }
        )
    }
}

pub struct Root;

impl Root {
    #[tracing::instrument]
    pub fn build(context: &Context, body: &Markup, active_page: Option<&TopLevelPage>) -> Markup {
        html!(
            (DOCTYPE)
            html {
                (Header::build())
                (Body::init(Parent::Root, BodyArgs {
                    body,
                    active_page
                }).build(&context))
            }
        )
    }
}
