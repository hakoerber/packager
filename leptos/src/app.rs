use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Script, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

#[cfg(feature = "ssr")]
pub use error::{
    AuthError, CommandError, DataError, DatabaseError, QueryError, RequestError, RunError,
    StartError,
};

#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
pub struct AppState {
    pub database_pool: database::Pool,
    pub auth_config: auth::Config,
}

#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
pub struct Context {
    user: user::User,
}

#[cfg(feature = "ssr")]
mod user {
    use uuid::Uuid;

    use super::error::{DatabaseError, RunError};

    #[derive(Debug, Clone)]
    pub struct User {
        pub id: Uuid,
        pub username: String,
        pub fullname: String,
    }

    #[derive(Debug)]
    pub struct NewUser<'a> {
        pub username: &'a str,
        pub fullname: &'a str,
    }

    #[derive(Debug)]
    pub struct DbUserRow {
        id: Uuid,
        username: String,
        fullname: String,
    }

    impl TryFrom<DbUserRow> for User {
        type Error = RunError;

        fn try_from(row: DbUserRow) -> Result<Self, Self::Error> {
            Ok(Self {
                id: row.id,
                username: row.username,
                fullname: row.fullname,
            })
        }
    }

    impl User {
        #[tracing::instrument]
        pub async fn find_by_name(
            pool: &database::Pool,
            name: &str,
        ) -> Result<Option<Self>, RunError> {
            database::query_one!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: super::Component::User,
                },
                pool,
                DbUserRow,
                Self,
                RunError,
                "SELECT id,username,fullname FROM users WHERE username = $1",
                name
            )
            .await
        }
    }

    #[tracing::instrument]
    pub async fn create(pool: &database::Pool, user: NewUser<'_>) -> Result<Uuid, DatabaseError> {
        let id = Uuid::new_v4();

        database::execute!(
            &database::QueryClassification {
                query_type: database::QueryType::Insert,
                component: super::Component::User,
            },
            pool,
            DatabaseError,
            "INSERT INTO users
            (id, username, fullname)
        VALUES
            ($1, $2, $3)",
            id,
            user.username,
            user.fullname
        )
        .await?;

        Ok(id)
    }
}

#[cfg(feature = "ssr")]
impl Context {
    fn build(user: user::User) -> Self {
        Self { user }
    }
}

pub enum Component {
    Inventory,
    User,
    Trips,
    Todo,
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Inventory => "inventory",
                Self::User => "user",
                Self::Trips => "trips",
                Self::Todo => "todo",
            }
        )
    }
}

#[cfg(feature = "ssr")]
mod error {
    use std::{convert::Infallible, fmt, net::SocketAddr};

    use axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
    };

    use leptos::prelude::*;

    pub use database::{Error as DatabaseError, QueryError};

    #[derive(Debug)]
    pub enum RequestError {
        EmptyFormElement { name: String },
        RefererNotFound,
        RefererInvalid { message: String },
        NotFound { message: String },
        Auth { inner: AuthError },
        Transport { inner: hyper::Error },
    }

    impl std::error::Error for RequestError {}

    impl fmt::Display for RequestError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::EmptyFormElement { name } => write!(f, "Form element {name} cannot be empty"),
                Self::RefererNotFound => write!(f, "Referer header not found"),
                Self::RefererInvalid { message } => write!(f, "Referer header invalid: {message}"),
                Self::NotFound { message } => write!(f, "Not found: {message}"),
                Self::Auth { inner } => {
                    write!(f, "Authentication failed: {inner}")
                }
                Self::Transport { inner } => {
                    write!(f, "HTTP error: {inner}")
                }
            }
        }
    }

    #[derive(Debug)]
    pub enum DataError {
        NotFound { description: String },
    }

    impl std::error::Error for DataError {}

    impl fmt::Display for DataError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::NotFound { description } => {
                    write!(f, "{description}")
                }
            }
        }
    }

    #[derive(Debug)]
    pub enum AuthError {
        AuthenticationUserNotFound { username: String },
        AuthenticationHeaderMissing,
        AuthenticationHeaderInvalid { message: String },
    }

    impl AuthError {
        #[must_use]
        pub fn to_prom_metric_name(&self) -> &'static str {
            match self {
                Self::AuthenticationUserNotFound { username: _ } => "user_not_found",
                Self::AuthenticationHeaderMissing => "header_missing",
                Self::AuthenticationHeaderInvalid { message: _ } => "header_invalid",
            }
        }

        pub fn trace(&self) {
            match self {
                Self::AuthenticationUserNotFound { username } => {
                    tracing::info!(username, "auth failed, user not found");
                }
                Self::AuthenticationHeaderMissing => {
                    tracing::info!("auth failed, auth header missing");
                }
                Self::AuthenticationHeaderInvalid { message } => {
                    tracing::info!(message, "auth failed, auth header invalid");
                }
            }
        }
    }

    impl<'a> AuthError {
        #[must_use]
        pub fn to_prom_labels(&'a self) -> Vec<(&'static str, String)> {
            match self {
                Self::AuthenticationUserNotFound { username } => {
                    vec![("username", username.clone())]
                }
                Self::AuthenticationHeaderMissing
                | Self::AuthenticationHeaderInvalid { message: _ } => vec![],
            }
        }
    }

    impl fmt::Display for AuthError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::AuthenticationUserNotFound { username } => {
                    write!(f, "User \"{username}\" not found")
                }
                Self::AuthenticationHeaderMissing => write!(f, "Authentication header not found"),
                Self::AuthenticationHeaderInvalid { message } => {
                    write!(f, "Authentication header invalid: {message}")
                }
            }
        }
    }

    impl From<AuthError> for RunError {
        fn from(e: AuthError) -> Self {
            Self::Request(RequestError::Auth { inner: e })
        }
    }

    impl std::error::Error for AuthError {}

    #[derive(Debug)]
    pub enum RunError {
        Request(RequestError),
        Database(database::Error),
        Data(DataError),
    }

    impl std::error::Error for RunError {}

    impl From<Infallible> for RunError {
        fn from(_value: Infallible) -> Self {
            unreachable!()
        }
    }

    impl fmt::Display for RunError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Request(request_error) => write!(f, "Request error: {request_error}"),
                Self::Database(db_error) => write!(f, "{db_error}"),
                Self::Data(data_error) => write!(f, "{data_error}"),
            }
        }
    }

    impl From<database::Error> for RunError {
        fn from(value: database::Error) -> Self {
            Self::Database(value)
        }
    }

    impl From<sqlx::Error> for RunError {
        fn from(value: sqlx::Error) -> Self {
            Self::Database(value.into())
        }
    }

    impl From<hyper::Error> for RunError {
        fn from(value: hyper::Error) -> Self {
            Self::Request(RequestError::Transport { inner: value })
        }
    }

    impl RunError {
        fn into_response(self) -> (StatusCode, impl IntoView) {
            match self {
                Self::Database(ref db_error) => match db_error {
                    database::Error::Database(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        super::errorpage::ErrorPage::build(&self.to_string()),
                    ),
                    database::Error::Query(error) => match error {
                        database::QueryError::NotFound { description } => (
                            StatusCode::NOT_FOUND,
                            super::errorpage::ErrorPage::build(description),
                        ),
                        _ => (
                            StatusCode::BAD_REQUEST,
                            super::errorpage::ErrorPage::build(&error.to_string()),
                        ),
                    },
                },
                Self::Request(request_error) => match request_error {
                    RequestError::RefererNotFound => (
                        StatusCode::BAD_REQUEST,
                        super::errorpage::ErrorPage::build("no referer header found"),
                    ),
                    RequestError::RefererInvalid { message } => (
                        StatusCode::BAD_REQUEST,
                        super::errorpage::ErrorPage::build(&format!(
                            "referer could not be converted: {message}"
                        )),
                    ),
                    RequestError::EmptyFormElement { name } => (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        super::errorpage::ErrorPage::build(&format!("empty form element: {name}")),
                    ),
                    RequestError::NotFound { message } => (
                        StatusCode::NOT_FOUND,
                        super::errorpage::ErrorPage::build(&format!("not found: {message}")),
                    ),
                    RequestError::Auth { inner: e } => (
                        StatusCode::UNAUTHORIZED,
                        super::errorpage::ErrorPage::build(&format!("authentication failed: {e}")),
                    ),
                    RequestError::Transport { inner } => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        super::errorpage::ErrorPage::build(&inner.to_string()),
                    ),
                },
                Self::Data(data_error) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    super::errorpage::ErrorPage::build(&data_error.to_string()),
                ),
            }
        }
    }

    #[derive(Debug)]
    pub enum StartError {
        Bind { addr: SocketAddr, message: String },
        Call { message: String },
        Exec(tokio::task::JoinError),
        DatabaseInit(database::InitError),
        AddrParse { input: String, message: String },
    }

    impl std::error::Error for StartError {}

    impl fmt::Display for StartError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Bind { message, addr } => {
                    write!(f, "error binding network interface {addr}: {message}")
                }
                Self::Call { message } => {
                    write!(f, "invalid invocation: {message}")
                }
                Self::Exec(join_error) => write!(f, "{join_error}"),
                Self::DatabaseInit(start_error) => write!(f, "{start_error}"),
                Self::AddrParse { message, input } => {
                    write!(f, "error parsing \"{input}\": {message}")
                }
            }
        }
    }

    impl From<tokio::task::JoinError> for StartError {
        fn from(value: tokio::task::JoinError) -> Self {
            Self::Exec(value)
        }
    }

    impl From<database::InitError> for StartError {
        fn from(value: database::InitError) -> Self {
            Self::DatabaseInit(value)
        }
    }

    impl From<(String, std::net::AddrParseError)> for StartError {
        fn from((input, error): (String, std::net::AddrParseError)) -> Self {
            Self::AddrParse {
                input,
                message: error.to_string(),
            }
        }
    }

    #[derive(Debug)]
    pub enum CommandError {
        Start(StartError),
        Database(database::Error),
        UserExists { username: String },
    }

    impl std::error::Error for CommandError {}

    impl fmt::Display for CommandError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Start(start_error) => {
                    write!(f, "{start_error}")
                }
                Self::Database(db_error) => write!(f, "{db_error}"),
                Self::UserExists { username } => {
                    write!(f, "user \"{username}\" already exists")
                }
            }
        }
    }

    impl From<tokio::task::JoinError> for CommandError {
        fn from(value: tokio::task::JoinError) -> Self {
            Self::Start(value.into())
        }
    }

    impl From<database::InitError> for CommandError {
        fn from(value: database::InitError) -> Self {
            Self::Start(value.into())
        }
    }

    impl From<(String, std::net::AddrParseError)> for CommandError {
        fn from((input, error): (String, std::net::AddrParseError)) -> Self {
            Self::Start(StartError::AddrParse {
                input,
                message: error.to_string(),
            })
        }
    }

    impl From<StartError> for CommandError {
        fn from(value: StartError) -> Self {
            Self::Start(value)
        }
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" />

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[derive(Debug, Serialize, Clone, Deserialize)]
/// Both ends are **inclusive**
pub struct TripDate {
    pub start: time::Date,
    pub end: time::Date,
}

impl fmt::Display for TripDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.start, self.end)
    }
}

#[cfg(feature = "ssr")]
#[expect(clippy::fallible_impl_from, reason = "panics only on buggy code")]
impl From<sqlx::postgres::types::PgRange<time::Date>> for TripDate {
    fn from(value: sqlx::postgres::types::PgRange<time::Date>) -> Self {
        Self {
            start: match value.start {
                core::ops::Bound::Included(d) => d,
                core::ops::Bound::Excluded(_d) => {
                    panic!("lower bound is exclusive, type contraint is wrong")
                }
                core::ops::Bound::Unbounded => {
                    panic!("lower bound missing, type contraint is wrong")
                }
            },
            end: match value.end {
                core::ops::Bound::Included(_d) => {
                    panic!("upper bound is inclusive, type contraint is wrong")
                }
                core::ops::Bound::Excluded(d) => d
                    .previous_day()
                    // this cannot even happen, as the range would be empty then
                    .expect("upper bound is Date::MIN, type contraint is wrong"),
                core::ops::Bound::Unbounded => {
                    panic!("lower bound missing, type contraint is wrong")
                }
            },
        }
    }
}

#[cfg(feature = "ssr")]
impl TryFrom<TripDate> for sqlx::postgres::types::PgRange<time::Date> {
    type Error = RunError;

    fn try_from(value: TripDate) -> Result<Self, RunError> {
        Ok(database::types::try_into_date_range(
            value.start,
            value.end,
        )?)
    }
}

#[cfg_attr(
    feature = "ssr",
    derive(sqlx::Type),
    sqlx(type_name = "trip_state"),
    sqlx(rename_all = "lowercase")
)]
#[derive(PartialEq, Eq, PartialOrd, Deserialize, Debug)]
pub enum TripState {
    Init,
    Planning,
    Planned,
    Active,
    Review,
    Done,
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
pub async fn trip_item_set_state(
    ctx: &Context,
    pool: &database::Pool,
    trip_id: Uuid,
    item_id: Uuid,
    key: TripItemStateKey,
    value: bool,
) -> Result<(), RunError> {
    TripItem::set_state(ctx, pool, trip_id, item_id, key, value).await
}

#[allow(clippy::new_without_default)]
impl TripState {
    #[must_use]
    pub fn new() -> Self {
        Self::Init
    }

    #[must_use]
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Init => Some(Self::Planning),
            Self::Planning => Some(Self::Planned),
            Self::Planned => Some(Self::Active),
            Self::Active => Some(Self::Review),
            Self::Review => Some(Self::Done),
            Self::Done => None,
        }
    }

    #[must_use]
    pub fn prev(&self) -> Option<Self> {
        match self {
            Self::Init => None,
            Self::Planning => Some(Self::Init),
            Self::Planned => Some(Self::Planning),
            Self::Active => Some(Self::Planned),
            Self::Review => Some(Self::Active),
            Self::Done => Some(Self::Review),
        }
    }
}

impl fmt::Display for TripState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Init => "Init",
                Self::Planning => "Planning",
                Self::Planned => "Planned",
                Self::Active => "Active",
                Self::Review => "Review",
                Self::Done => "Done",
            },
        )
    }
}

#[cfg(feature = "ssr")]
impl std::convert::TryFrom<&str> for TripState {
    type Error = RunError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(database::types::try_into_enum(
            value,
            |value| match value {
                "Init" => Some(Self::Init),
                "Planning" => Some(Self::Planning),
                "Planned" => Some(Self::Planned),
                "Active" => Some(Self::Active),
                "Review" => Some(Self::Review),
                "Done" => Some(Self::Done),
                _ => None,
            },
        )?)
    }
}

#[derive(Serialize, Debug)]
pub enum TripItemStateKey {
    Pick,
    Pack,
    Ready,
}

impl fmt::Display for TripItemStateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Pick => "pick",
                Self::Pack => "pack",
                Self::Ready => "ready",
            },
        )
    }
}

#[derive(Debug)]
pub struct TripCategory {
    pub category: inventory::Category,
    pub items: Option<Vec<TripItem>>,
}

impl TripCategory {
    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub fn total_picked_weight(&self) -> i32 {
        self.items
            .as_ref()
            .unwrap()
            .iter()
            .filter(|item| item.picked)
            .map(|item| item.item.weight)
            .sum()
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn find(
        ctx: &Context,
        pool: &database::Pool,
        trip_id: Uuid,
        category_id: Uuid,
    ) -> Result<Option<Self>, RunError> {
        struct Row {
            category_id: Uuid,
            category_name: String,
            #[allow(dead_code)]
            trip_id: Option<Uuid>,
            item_id: Option<Uuid>,
            item_name: Option<String>,
            item_description: Option<String>,
            item_weight: Option<i32>,
            item_is_picked: Option<bool>,
            item_is_packed: Option<bool>,
            item_is_ready: Option<bool>,
            item_is_new: Option<bool>,
        }

        struct RowParsed {
            category: TripCategory,
            item: Option<TripItem>,
        }

        impl TryFrom<Row> for RowParsed {
            type Error = RunError;

            fn try_from(row: Row) -> Result<Self, Self::Error> {
                let category = inventory::Category {
                    id: row.category_id,
                    name: row.category_name,
                    items: None,
                };
                Ok(Self {
                    category: TripCategory {
                        category,
                        items: None,
                    },

                    item: match row.item_id {
                        Some(item_id) => Some(TripItem {
                            item: inventory::Item {
                                id: item_id,
                                name: row.item_name.unwrap(),
                                description: row.item_description,
                                weight: row.item_weight.unwrap(),
                                category_id: row.category_id,
                            },
                            picked: row.item_is_picked.unwrap(),
                            packed: row.item_is_packed.unwrap(),
                            ready: row.item_is_ready.unwrap(),
                            new: row.item_is_new.unwrap(),
                        }),
                        None => None,
                    },
                })
            }
        }

        let mut rows = database::query_all!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: Component::Trips,
            },
            pool,
            Row,
            RowParsed,
            RunError,
            r"
                WITH category_items AS (
                     SELECT
                        trip.trip_id AS trip_id,
                        category.id AS category_id,
                        category.name AS category_name,
                        item.id AS item_id,
                        item.name AS item_name,
                        item.description AS item_description,
                        item.weight AS item_weight,
                        trip.pick AS item_is_picked,
                        trip.pack AS item_is_packed,
                        trip.ready AS item_is_ready,
                        trip.new AS item_is_new
                    FROM trip_items AS trip
                    INNER JOIN inventory_items AS item
                        ON item.id = trip.item_id
                    INNER JOIN inventory_items_categories AS category
                        ON category.id = item.category_id
                    WHERE
                        trip.trip_id = $1
                        AND trip.user_id = $2
                )
                SELECT
                    category.id AS category_id,
                    category.name AS category_name,
                    items.trip_id AS trip_id,
                    items.item_id AS item_id,
                    items.item_name AS item_name,
                    items.item_description AS item_description,
                    items.item_weight AS item_weight,
                    items.item_is_picked AS item_is_picked,
                    items.item_is_packed AS item_is_packed,
                    items.item_is_ready AS item_is_ready,
                    items.item_is_new AS item_is_new
                FROM inventory_items_categories AS category
                    LEFT JOIN category_items AS items
                    ON items.category_id = category.id
                WHERE category.id = $3
            ",
            trip_id,
            ctx.user.id,
            category_id
        )
        .await?;

        let mut category = match rows.pop() {
            None => return Ok(None),
            Some(initial) => Self {
                category: initial.category.category,
                items: initial.item.map(|item| vec![item]).or_else(|| Some(vec![])),
            },
        };

        for row in rows {
            let item = row.item;
            category.items = category.items.or_else(|| Some(vec![]));

            if let Some(item) = item {
                category.items = category.items.map(|mut c| {
                    c.push(item);
                    c
                });
            }
        }

        Ok(Some(category))
    }
}

// TODO refactor the bools into an enum
#[derive(Debug)]
pub struct TripItem {
    pub item: inventory::Item,
    pub picked: bool,
    pub packed: bool,
    pub ready: bool,
    pub new: bool,
}

pub struct DbTripsItemsRow {
    pub picked: bool,
    pub packed: bool,
    pub ready: bool,
    pub new: bool,
    pub id: Uuid,
    pub name: String,
    pub weight: i32,
    pub description: Option<String>,
    pub category_id: Uuid,
}

#[cfg(feature = "ssr")]
impl TryFrom<DbTripsItemsRow> for TripItem {
    type Error = RunError;

    fn try_from(row: DbTripsItemsRow) -> Result<Self, Self::Error> {
        Ok(Self {
            picked: row.picked,
            packed: row.packed,
            ready: row.ready,
            new: row.new,
            item: inventory::Item {
                id: row.id,
                name: row.name,
                description: row.description,
                weight: row.weight,
                category_id: row.category_id,
            },
        })
    }
}

impl TripItem {
    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn find(
        ctx: &Context,
        pool: &database::Pool,
        trip_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<Self>, RunError> {
        database::query_one!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: Component::Trips,
            },
            pool,
            DbTripsItemsRow,
            Self,
            RunError,
            "
                SELECT
                    t_item.item_id AS id,
                    t_item.pick AS picked,
                    t_item.pack AS packed,
                    t_item.ready AS ready,
                    t_item.new AS new,
                    i_item.name AS name,
                    i_item.description AS description,
                    i_item.weight AS weight,
                    i_item.category_id AS category_id
                FROM trip_items AS t_item
                INNER JOIN inventory_items AS i_item
                    ON i_item.id = t_item.item_id
                WHERE t_item.item_id = $1
                AND t_item.trip_id = $2
                AND t_item.user_id = $3
            ",
            item_id,
            trip_id,
            ctx.user.id
        )
        .await
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn set_state(
        ctx: &Context,
        pool: &database::Pool,
        trip_id: Uuid,
        item_id: Uuid,
        key: TripItemStateKey,
        value: bool,
    ) -> Result<(), RunError> {
        let result = match key {
            TripItemStateKey::Pick => {
                database::execute!(
                    &database::QueryClassification {
                        query_type: database::QueryType::Update,
                        component: Component::Trips,
                    },
                    pool,
                    RunError,
                    "UPDATE trip_items
                        SET pick = $1
                        WHERE trip_id = $2
                        AND item_id = $3
                        AND user_id = $4",
                    value,
                    trip_id,
                    item_id,
                    ctx.user.id
                )
                .await
            }
            TripItemStateKey::Pack => {
                database::execute!(
                    &database::QueryClassification {
                        query_type: database::QueryType::Update,
                        component: Component::Trips,
                    },
                    pool,
                    RunError,
                    "UPDATE trip_items
                        SET pack = $1
                        WHERE trip_id = $2
                        AND item_id = $3
                        AND user_id = $4",
                    value,
                    trip_id,
                    item_id,
                    ctx.user.id
                )
                .await
            }
            TripItemStateKey::Ready => {
                database::execute!(
                    &database::QueryClassification {
                        query_type: database::QueryType::Update,
                        component: Component::Trips,
                    },
                    pool,
                    RunError,
                    "UPDATE trip_items
                        SET ready = $1
                        WHERE trip_id = $2
                        AND item_id = $3
                        AND user_id = $4",
                    value,
                    trip_id,
                    item_id,
                    ctx.user.id
                )
                .await
            }
        }?;

        (result.rows_affected() != 0).then_some(()).ok_or_else(|| {
            RunError::Data(DataError::NotFound {
                description: format!("item {item_id} not found for trip {trip_id}"),
            })
        })
    }
}

pub struct DbTripRow {
    pub id: Uuid,
    pub name: String,
    pub date: TripDate,
    pub state: TripState,
    pub location: Option<String>,
    pub temp_min: Option<i32>,
    pub temp_max: Option<i32>,
    pub comment: Option<String>,
}

#[cfg(feature = "ssr")]
impl TryFrom<DbTripRow> for Trip {
    type Error = RunError;

    fn try_from(row: DbTripRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            name: row.name,
            date: row.date,
            state: row.state,
            location: row.location,
            temp_min: row.temp_min,
            temp_max: row.temp_max,
            comment: row.comment,
            todos: None,
            types: None,
            categories: None,
        })
    }
}

#[derive(Debug)]
pub struct Trip {
    pub id: Uuid,
    pub name: String,
    pub date: TripDate,
    pub state: TripState,
    pub location: Option<String>,
    pub temp_min: Option<i32>,
    pub temp_max: Option<i32>,
    pub comment: Option<String>,
    pub todos: Option<Vec<todo::Todo>>,
    pub types: Option<Vec<TripType>>,
    pub categories: Option<Vec<TripCategory>>,
}

impl Trip {
    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn all(ctx: &Context, pool: &database::Pool) -> Result<Vec<Self>, RunError> {
        let mut trips = database::query_all!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: Component::Trips,
            },
            pool,
            DbTripRow,
            Self,
            RunError,
            r#"SELECT
                id,
                name,
                date,
                state as "state: _",
                location,
                temp_min,
                temp_max,
                comment
            FROM trips
            WHERE user_id = $1"#,
            ctx.user.id
        )
        .await?;

        trips.sort_by_key(|trip| trip.date.start);
        Ok(trips)
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn find(
        ctx: &Context,
        pool: &database::Pool,
        trip_id: Uuid,
    ) -> Result<Option<Self>, RunError> {
        database::query_one!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: Component::Trips,
            },
            pool,
            DbTripRow,
            Self,
            RunError,
            r#"SELECT
                id,
                name,
                date,
                state as "state: _",
                location,
                temp_min,
                temp_max,
                comment
            FROM trips
            WHERE id = $1 and user_id = $2"#,
            trip_id,
            ctx.user.id
        )
        .await
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn trip_type_remove(
        ctx: &Context,
        pool: &database::Pool,
        id: Uuid,
        type_id: Uuid,
    ) -> Result<bool, RunError> {
        let results = database::execute!(
            &database::QueryClassification {
                query_type: database::QueryType::Delete,
                component: Component::Trips,
            },
            pool,
            RunError,
            "DELETE FROM trip_to_trip_types AS ttt
            WHERE ttt.trip_id = $1
                AND ttt.trip_type_id = $2
            AND EXISTS(SELECT * FROM trips WHERE id = $1 AND user_id = $3)
            AND EXISTS(SELECT * FROM trip_types WHERE id = $2 AND user_id = $3)
            ",
            id,
            type_id,
            ctx.user.id,
        )
        .await?;

        Ok(results.rows_affected() != 0)
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn trip_type_add(
        ctx: &Context,
        pool: &database::Pool,
        id: Uuid,
        type_id: Uuid,
    ) -> Result<(), RunError> {
        // TODO user handling?

        database::execute!(
            &database::QueryClassification {
                query_type: database::QueryType::Insert,
                component: Component::Trips,
            },
            pool,
            RunError,
            "INSERT INTO
                trip_to_trip_types (trip_id, trip_type_id)
            (SELECT trips.id as trip_id, trip_types.id as trip_type_id
                FROM trips
                INNER JOIN trip_types ON true
                WHERE
                    trips.id = $1
                    AND trips.user_id = $3
                    AND trip_types.id = $2
                    AND trip_types.user_id = $3)",
            id,
            type_id,
            ctx.user.id
        )
        .await?;

        Ok(())
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn set_state(
        ctx: &Context,
        pool: &database::Pool,
        id: Uuid,
        new_state: &TripState,
    ) -> Result<bool, RunError> {
        let result = database::execute!(
            &database::QueryClassification {
                query_type: database::QueryType::Update,
                component: Component::Trips,
            },
            pool,
            RunError,
            "UPDATE trips
            SET state = $1
            WHERE id = $2 and user_id = $3",
            new_state as _,
            id,
            ctx.user.id
        )
        .await?;

        Ok(result.rows_affected() != 0)
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn set_comment(
        ctx: &Context,
        pool: &database::Pool,
        id: Uuid,
        new_comment: &str,
    ) -> Result<bool, RunError> {
        let result = database::execute!(
            &database::QueryClassification {
                query_type: database::QueryType::Update,
                component: Component::Trips,
            },
            pool,
            RunError,
            "UPDATE trips
            SET comment = $1
            WHERE id = $2 AND user_id = $3",
            new_comment,
            id,
            ctx.user.id
        )
        .await?;

        Ok(result.rows_affected() != 0)
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn save(
        ctx: &Context,
        pool: &database::Pool,
        name: &str,
        date: TripDate,
        copy_from: Option<Uuid>,
    ) -> Result<Uuid, RunError> {
        let id = Uuid::new_v4();

        let trip_state = TripState::new();

        let mut transaction = pool.begin().await?;

        println!("date: {date:?}");

        database::execute!(
            &database::QueryClassification {
                query_type: database::QueryType::Insert,
                component: Component::Trips,
            },
            &mut *transaction,
            RunError,
            "INSERT INTO trips
                (id, name, date, state, user_id)
            VALUES
                ($1, $2, $3, $4, $5)",
            id,
            name,
            <TripDate as TryInto<sqlx::postgres::types::PgRange<time::Date>>>::try_into(date)?,
            trip_state as _,
            ctx.user.id,
        )
        .await?;

        if let Some(copy_from_trip_id) = copy_from {
            database::execute!(
                &database::QueryClassification {
                    query_type: database::QueryType::Insert,
                    component: Component::Trips,
                },
                &mut *transaction,
                RunError,
                r"INSERT INTO trip_items (
                    item_id,
                    trip_id,
                    pick,
                    pack,
                    ready,
                    new,
                    user_id
                ) SELECT
                    item_id,
                    $1 as trip_id,
                    pick,
                    false as pack,
                    false as ready,
                    false as new,
                    user_id
                FROM trip_items
                WHERE trip_id = $2 AND user_id = $3",
                id,
                copy_from_trip_id,
                ctx.user.id
            )
            .await?;
        } else {
            database::execute!(
                &database::QueryClassification {
                    query_type: database::QueryType::Insert,
                    component: Component::Trips,
                },
                &mut *transaction,
                RunError,
                r"INSERT INTO trip_items (
                    item_id,
                    trip_id,
                    pick,
                    pack,
                    ready,
                    new,
                    user_id
                ) SELECT
                    id as item_id,
                    $1 as trip_id,
                    false as pick,
                    false as pack,
                    false as ready,
                    false as new,
                    user_id
                FROM inventory_items
                WHERE user_id = $2",
                id,
                ctx.user.id
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(id)
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn find_total_picked_weight(
        ctx: &Context,
        pool: &database::Pool,
        trip_id: Uuid,
    ) -> Result<i32, RunError> {
        let weight = database::execute_returning!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: Component::Trips,
            },
            pool,
            RunError,
            "
                SELECT
                    CAST(COALESCE(SUM(i_item.weight), 0) AS INTEGER) AS total_weight
                FROM trips AS trip
                INNER JOIN trip_items AS t_item
                    ON t_item.trip_id = trip.id
                INNER JOIN inventory_items AS i_item
                    ON t_item.item_id = i_item.id
                WHERE
                    trip.id = $1 AND trip.user_id = $2
                AND t_item.pick = true
            ",
            i32,
            |row| row.total_weight.unwrap(),
            trip_id,
            ctx.user.id
        )
        .await?;

        Ok(weight)
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub fn types(&self) -> &Vec<TripType> {
        self.types
            .as_ref()
            .expect("you need to call load_trip_types()")
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub fn categories(&self) -> &Vec<TripCategory> {
        self.categories
            .as_ref()
            .expect("you need to call load_categories()")
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub fn todos(&self) -> &Vec<todo::Todo> {
        self.todos.as_ref().expect("you need to call load_todos()")
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub fn total_picked_weight(&self) -> i32 {
        self.categories()
            .iter()
            .map(|category| -> i32 {
                category
                    .items
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter_map(|item| Some(item.item.weight).filter(|_| item.picked))
                    .sum::<i32>()
            })
            .sum::<i32>()
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn load_todos(
        &mut self,
        ctx: &Context,
        pool: &database::Pool,
    ) -> Result<(), RunError> {
        self.todos =
            Some(todo::Todo::findall(ctx, pool, todo::Container { trip_id: self.id }).await?);
        Ok(())
    }

    #[cfg(feature = "ssr")]
    #[tracing::instrument]
    pub async fn load_trip_types(
        &mut self,
        ctx: &Context,
        pool: &database::Pool,
    ) -> Result<(), RunError> {
        let types = database::query_all!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: Component::Trips,
            },
            pool,
            TripTypeRow,
            TripType,
            RunError,
            r#"
            WITH trips AS (
                SELECT type.id as id, trip.user_id as user_id
                FROM trips as trip
                INNER JOIN trip_to_trip_types as ttt
                    ON ttt.trip_id = trip.id
                INNER JOIN trip_types AS type
                    ON type.id = ttt.trip_type_id
                WHERE trip.id = $1 AND trip.user_id = $2
            )
            SELECT
                type.id AS id,
                type.name AS name,
                trips.id IS NOT NULL AS "active!"
            FROM trip_types AS type
                LEFT JOIN trips
                ON trips.id = type.id
            WHERE type.user_id = $2
            "#,
            self.id,
            ctx.user.id
        )
        .await?;

        self.types = Some(types);
        Ok(())
    }

    #[tracing::instrument]
    pub async fn sync_trip_items_with_inventory(
        &self,
        ctx: &Context,
        pool: &database::Pool,
    ) -> Result<(), RunError> {
        // we need to get all items that are part of the inventory but not
        // part of the trip items
        //
        // then, we know which items we need to sync. there are different
        // states for them:
        //
        // * if the trip is new (it's state is INITIAL), we can just forward
        //   as-is
        // * if the trip is not new, we have to make these new items prominently
        //   visible so the user knows that there might be new items to
        //   consider
        struct Row {
            item_id: Uuid,
        }

        impl TryFrom<Row> for Uuid {
            type Error = RunError;

            fn try_from(value: Row) -> Result<Self, Self::Error> {
                Ok(value.item_id)
            }
        }

        let unsynced_items: Vec<Uuid> = database::query_all!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: Component::Trips,
            },
            pool,
            Row,
            Uuid,
            RunError,
            "
            SELECT
                i_item.id AS item_id
            FROM inventory_items AS i_item
                LEFT JOIN (
                    SELECT t_item.item_id AS item_id, t_item.user_id AS user_id
                    FROM trip_items AS t_item
                    WHERE t_item.trip_id = $1 AND t_item.user_id = $2
                ) AS t_item
                ON t_item.item_id = i_item.id
            WHERE t_item.item_id IS NULL AND i_item.user_id = $2",
            self.id,
            ctx.user.id
        )
        .await?;

        // only mark as new when the trip not already underway
        let mark_as_new = self.state < TripState::Active;

        for unsynced_item in &unsynced_items {
            database::execute!(
                &database::QueryClassification {
                    query_type: database::QueryType::Insert,
                    component: Component::Trips,
                },
                pool,
                RunError,
                "
                INSERT INTO trip_items
                    (
                        item_id,
                        trip_id,
                        pick,
                        pack,
                        ready,
                        new,
                        user_id
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                ",
                unsynced_item,
                self.id,
                false,
                false,
                false,
                mark_as_new,
                ctx.user.id
            )
            .await?;
        }

        tracing::info!("unsynced items: {:?}", &unsynced_items);

        Ok(())
    }

    #[tracing::instrument]
    pub async fn load_categories(
        &mut self,
        ctx: &Context,
        pool: &database::Pool,
    ) -> Result<(), RunError> {
        let mut categories: Vec<TripCategory> = vec![];
        // we can ignore the return type as we collect into `categories`
        // in the `map_ok()` closure
        struct Row {
            category_id: Uuid,
            category_name: String,
            #[allow(dead_code)]
            trip_id: Option<Uuid>,
            item_id: Option<Uuid>,
            item_name: Option<String>,
            item_description: Option<String>,
            item_weight: Option<i32>,
            item_is_picked: Option<bool>,
            item_is_packed: Option<bool>,
            item_is_ready: Option<bool>,
            item_is_new: Option<bool>,
        }

        struct RowParsed {
            category: TripCategory,
            item: Option<TripItem>,
        }

        impl TryFrom<Row> for RowParsed {
            type Error = RunError;

            fn try_from(row: Row) -> Result<Self, Self::Error> {
                let category = inventory::Category {
                    id: row.category_id,
                    name: row.category_name,
                    items: None,
                };
                Ok(Self {
                    category: TripCategory {
                        category,
                        items: None,
                    },

                    item: match row.item_id {
                        Some(item_id) => Some(TripItem {
                            item: inventory::Item {
                                id: item_id,
                                name: row.item_name.unwrap(),
                                description: row.item_description,
                                weight: row.item_weight.unwrap(),
                                category_id: row.category_id,
                            },
                            picked: row.item_is_picked.unwrap(),
                            packed: row.item_is_packed.unwrap(),
                            ready: row.item_is_ready.unwrap(),
                            new: row.item_is_new.unwrap(),
                        }),
                        None => None,
                    },
                })
            }
        }

        let rows = database::query_all!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: Component::Trips,
            },
            pool,
            Row,
            RowParsed,
            RunError,
            r"
                WITH trip_items AS (
                    SELECT
                        trip.trip_id AS trip_id,
                        category.id AS category_id,
                        category.name AS category_name,
                        item.id AS item_id,
                        item.name AS item_name,
                        item.description AS item_description,
                        item.weight AS item_weight,
                        trip.pick AS item_is_picked,
                        trip.pack AS item_is_packed,
                        trip.ready AS item_is_ready,
                        trip.new AS item_is_new,
                        trip.user_id AS user_id
                    FROM trip_items AS trip
                    INNER JOIN inventory_items AS item
                        ON item.id = trip.item_id
                    INNER JOIN inventory_items_categories AS category
                        ON category.id = item.category_id
                    WHERE trip.trip_id = $1 AND trip.user_id = $2
                )
                SELECT
                    category.id AS category_id,
                    category.name AS category_name,
                    trip_items.trip_id AS trip_id,
                    trip_items.item_id AS item_id,
                    trip_items.item_name AS item_name,
                    trip_items.item_description AS item_description,
                    trip_items.item_weight AS item_weight,
                    trip_items.item_is_picked AS item_is_picked,
                    trip_items.item_is_packed AS item_is_packed,
                    trip_items.item_is_ready AS item_is_ready,
                    trip_items.item_is_new AS item_is_new
                FROM inventory_items_categories AS category
                    LEFT JOIN trip_items
                    ON trip_items.category_id = category.id
                WHERE category.user_id = $2
            ",
            self.id,
            ctx.user.id
        )
        .await?;

        for row in rows {
            match categories
                .iter_mut()
                .find(|cat| cat.category.id == row.category.category.id)
            {
                Some(ref mut existing_category) => {
                    // taking and then readding later
                    let mut items = existing_category.items.take().unwrap_or(vec![]);

                    if let Some(item) = row.item {
                        items.push(item);
                    }

                    existing_category.items = Some(items);
                }
                None => categories.push(TripCategory {
                    category: row.category.category,
                    items: row.item.map(|item| vec![item]).or_else(|| Some(vec![])),
                }),
            }
        }

        self.categories = Some(categories);

        Ok(())
        // .fetch(pool)
        // .map_ok(|row| -> Result<(), Error> {
        //     let mut category = TripCategory {
        //         category: inventory::Category {
        //             id: Uuid::try_parse(&row.category_id)?,
        //             name: row.category_name,
        //             description: row.category_description,

        //             items: None,
        //         },
        //         items: None,
        //     };

        //     match row.item_id {
        //         None => {
        //             // we have an empty (unused) category which has NULL values
        //             // for the item_id column
        //             category.items = Some(vec![]);
        //             categories.push(category);
        //         }
        //         Some(item_id) => {
        //             let item = TripItem {
        //                 item: inventory::Item {
        //                     id: Uuid::try_parse(&item_id)?,
        //                     name: row.item_name.unwrap(),
        //                     description: row.item_description,
        //                     weight: row.item_weight.unwrap(),
        //                     category_id: category.category.id,
        //                 },
        //                 picked: row.item_is_picked.unwrap(),
        //                 packed: row.item_is_packed.unwrap(),
        //                 ready: row.item_is_ready.unwrap(),
        //                 new: row.item_is_new.unwrap(),
        //             };

        //             if let Some(&mut ref mut c) = categories
        //                 .iter_mut()
        //                 .find(|c| c.category.id == category.category.id)
        //             {
        //                 // we always populate c.items when we add a new category, so
        //                 // it's safe to unwrap here
        //                 c.items.as_mut().unwrap().push(item);
        //             } else {
        //                 category.items = Some(vec![item]);
        //                 categories.push(category);
        //             }
        //         }
        //     }

        //     Ok(())
        // })
        // .try_collect::<Vec<Result<(), Error>>>()
        // .await?
        // .into_iter()
        // .collect::<Result<(), Error>>()?;
    }
}

#[derive(Debug)]
pub struct TripType {
    pub id: Uuid,
    pub name: String,
    pub active: bool,
}

struct TripTypeRow {
    id: Uuid,
    name: String,
    active: bool,
}

impl TryFrom<TripTypeRow> for TripType {
    type Error = RunError;

    fn try_from(row: TripTypeRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            name: row.name,
            active: row.active,
        })
    }
}

impl TripsType {
    #[tracing::instrument]
    pub async fn all(ctx: &Context, pool: &database::Pool) -> Result<Vec<Self>, RunError> {
        database::query_all!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: Component::Trips,
            },
            pool,
            DbTripsTypesRow,
            Self,
            RunError,
            "SELECT
                id,
                name
            FROM trip_types
            WHERE user_id = $1",
            ctx.user.id
        )
        .await
    }

    #[tracing::instrument]
    pub async fn save(ctx: &Context, pool: &database::Pool, name: &str) -> Result<Uuid, RunError> {
        let id = Uuid::new_v4();
        database::execute!(
            &database::QueryClassification {
                query_type: database::QueryType::Insert,
                component: Component::Trips,
            },
            pool,
            RunError,
            "INSERT INTO trip_types
                (id, name, user_id)
            VALUES
                ($1, $2, $3)",
            id,
            name,
            ctx.user.id
        )
        .await?;

        Ok(id)
    }

    #[tracing::instrument]
    pub async fn set_name(
        ctx: &Context,
        pool: &database::Pool,
        id: Uuid,
        new_name: &str,
    ) -> Result<bool, RunError> {
        let result = database::execute!(
            &database::QueryClassification {
                query_type: database::QueryType::Update,
                component: Component::Trips,
            },
            pool,
            RunError,
            "UPDATE trip_types
            SET name = $1
            WHERE id = $2 and user_id = $3",
            new_name,
            id,
            ctx.user.id
        )
        .await?;

        Ok(result.rows_affected() != 0)
    }
}

pub struct DbTripsTypesRow {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TripTypeAttribute {
    #[serde(rename = "name")]
    Name,
}

#[derive(Debug)]
pub struct TripsType {
    pub id: Uuid,
    pub name: String,
}

impl TryFrom<DbTripsTypesRow> for TripsType {
    type Error = RunError;

    fn try_from(row: DbTripsTypesRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            name: row.name,
        })
    }
}

mod inventory {
    use super::{Context, RunError};

    use uuid::Uuid;

    #[derive(Debug)]
    pub struct Product {
        #[allow(dead_code)]
        pub id: Uuid,
        pub name: String,
        #[allow(dead_code)]
        pub description: Option<String>,
    }

    pub struct Inventory {
        pub categories: Vec<Category>,
    }

    impl Inventory {
        #[tracing::instrument]
        pub async fn load(ctx: &Context, pool: &database::Pool) -> Result<Self, RunError> {
            let mut categories = database::query_all!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: super::Component::Inventory,
                },
                pool,
                DbCategoryRow,
                Category,
                RunError,
                "SELECT
                    id,
                    name
                FROM inventory_items_categories
                WHERE user_id = $1",
                ctx.user.id
            )
            .await?;

            for category in &mut categories {
                category.populate_items(ctx, pool).await?;
            }

            Ok(Self { categories })
        }
    }

    #[derive(Debug)]
    pub struct Category {
        pub id: Uuid,
        pub name: String,
        pub items: Option<Vec<Item>>,
    }

    pub struct DbCategoryRow {
        pub id: Uuid,
        pub name: String,
    }

    impl TryFrom<DbCategoryRow> for Category {
        type Error = RunError;

        fn try_from(row: DbCategoryRow) -> Result<Self, Self::Error> {
            Ok(Self {
                id: row.id,
                name: row.name,
                items: None,
            })
        }
    }

    impl Category {
        #[tracing::instrument]
        pub async fn _find(
            ctx: &Context,
            pool: &database::Pool,
            id: Uuid,
        ) -> Result<Option<Self>, RunError> {
            database::query_one!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: super::Component::Inventory,
                },
                pool,
                DbCategoryRow,
                Category,
                RunError,
                "SELECT
                id,
                name
            FROM inventory_items_categories AS category
            WHERE
                category.id = $1
                AND category.user_id = $2",
                id,
                ctx.user.id,
            )
            .await
        }

        #[tracing::instrument]
        pub async fn save(
            ctx: &Context,
            pool: &database::Pool,
            name: &str,
        ) -> Result<Uuid, RunError> {
            let id = Uuid::new_v4();
            database::execute!(
                &database::QueryClassification {
                    query_type: database::QueryType::Insert,
                    component: super::Component::Inventory,
                },
                pool,
                RunError,
                "INSERT INTO inventory_items_categories
                (id, name, user_id)
            VALUES
                ($1, $2, $3)",
                id,
                name,
                ctx.user.id,
            )
            .await?;

            Ok(id)
        }

        #[tracing::instrument]
        pub fn items(&self) -> &Vec<Item> {
            self.items
                .as_ref()
                .expect("you need to call populate_items()")
        }

        #[tracing::instrument]
        pub fn total_weight(&self) -> i32 {
            self.items().iter().map(|item| item.weight).sum()
        }

        #[tracing::instrument]
        pub async fn populate_items(
            &mut self,
            ctx: &Context,
            pool: &database::Pool,
        ) -> Result<(), RunError> {
            let items = database::query_all!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: super::Component::Inventory,
                },
                pool,
                DbInventoryItemsRow,
                Item,
                RunError,
                "SELECT
                id,
                name,
                weight,
                description,
                category_id
            FROM inventory_items
            WHERE
                category_id = $1
                AND user_id = $2",
                self.id,
                ctx.user.id,
            )
            .await?;

            self.items = Some(items);
            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct InventoryItemTrip {
        pub name: String,
        // pub date: crate::domains::trips::TripDate,
        pub state: super::TripState,
    }

    #[derive(Debug)]
    struct DbInventoryItemRows {
        first: DbInventoryItemRow,
        rest: Vec<DbInventoryItemRow>,
    }

    impl DbInventoryItemRows {
        fn first(&self) -> &DbInventoryItemRow {
            &self.first
        }
    }

    impl<'a> DbInventoryItemRows {
        #[allow(dead_code)]
        fn iter(&'a self) -> DbInventoryItemRowsIterRef<'a> {
            DbInventoryItemRowsIterRef {
                first: Some(&self.first),
                inner_iter: self.rest.iter(),
            }
        }

        fn iter_mut(&'a mut self) -> DbInventoryItemRowsIterRefMut<'a> {
            DbInventoryItemRowsIterRefMut {
                first: Some(&mut self.first),
                inner_iter: self.rest.iter_mut(),
            }
        }
    }

    #[allow(dead_code)]
    struct DbInventoryItemRowsIterRef<'a> {
        first: Option<&'a DbInventoryItemRow>,
        inner_iter: std::slice::Iter<'a, DbInventoryItemRow>,
    }

    impl<'a> Iterator for DbInventoryItemRowsIterRef<'a> {
        type Item = &'a DbInventoryItemRow;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(first) = self.first.take() {
                Some(first)
            } else {
                self.inner_iter.next()
            }
        }
    }

    struct DbInventoryItemRowsIterRefMut<'a> {
        first: Option<&'a mut DbInventoryItemRow>,
        inner_iter: std::slice::IterMut<'a, DbInventoryItemRow>,
    }

    impl<'a> Iterator for DbInventoryItemRowsIterRefMut<'a> {
        type Item = &'a mut DbInventoryItemRow;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(first) = self.first.take() {
                Some(first)
            } else {
                self.inner_iter.next()
            }
        }
    }

    struct DbInventoryItemRowsIter {
        first: Option<DbInventoryItemRow>,
        inner_iter: std::vec::IntoIter<DbInventoryItemRow>,
    }

    impl Iterator for DbInventoryItemRowsIter {
        type Item = DbInventoryItemRow;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(first) = self.first.take() {
                Some(first)
            } else {
                self.inner_iter.next()
            }
        }
    }

    impl IntoIterator for DbInventoryItemRows {
        type Item = DbInventoryItemRow;

        type IntoIter = DbInventoryItemRowsIter;

        fn into_iter(self) -> Self::IntoIter {
            Self::IntoIter {
                first: Some(self.first),
                inner_iter: self.rest.into_iter(),
            }
        }
    }

    #[expect(clippy::fallible_impl_from, reason = "panics only on buggy code")]
    impl From<Vec<DbInventoryItemRow>> for DbInventoryItemRows {
        fn from(mut value: Vec<DbInventoryItemRow>) -> Self {
            match value.pop() {
                Some(first) => Self { first, rest: value },
                None => panic!("received empty vec, this is a bug"),
            }
        }
    }

    #[derive(Debug)]
    struct DbInventoryItemRow {
        pub id: Uuid,
        pub name: String,
        pub description: Option<String>,
        pub weight: i32,
        pub category_id: Uuid,
        pub category_name: String,
        pub product_id: Option<Uuid>,
        pub product_name: Option<String>,
        pub product_description: Option<String>,
        pub trip_name: Option<String>,
        // pub trip_date: Option<crate::domains::trips::TripDate>,
        pub trip_state: Option<super::TripState>,
    }

    #[derive(Debug)]
    pub struct InventoryItem {
        #[allow(dead_code)]
        pub id: Uuid,
        pub name: String,
        pub description: Option<String>,
        pub weight: i32,
        pub category: Category,
        pub product: Option<Product>,
        pub trips: Vec<InventoryItemTrip>,
    }

    impl TryFrom<DbInventoryItemRows> for InventoryItem {
        type Error = RunError;

        fn try_from(mut rows: DbInventoryItemRows) -> Result<Self, Self::Error> {
            let first_id = rows.first().id;

            let mut trips: Vec<InventoryItemTrip> = vec![];

            for row in rows.iter_mut() {
                assert_eq!(row.id, first_id);
                if let Some(name) = row.trip_name.take() {
                    // safe because trip_id is non-NULL
                    let state = row.trip_state.take().unwrap();
                    trips.push(InventoryItemTrip { name, state });
                }
            }

            let item = rows.first;

            Ok(Self {
                id: item.id,
                name: item.name,
                description: item.description,
                weight: item.weight,
                category: Category {
                    id: item.category_id,
                    name: item.category_name,
                    items: None,
                },
                product: item
                    .product_id
                    .map(|id| -> Result<Product, RunError> {
                        Ok(Product {
                            id,
                            name: item.product_name.unwrap(),
                            description: item.product_description,
                        })
                    })
                    .transpose()?,
                trips,
            })
        }
    }

    impl InventoryItem {
        #[tracing::instrument]
        pub async fn find(
            ctx: &Context,
            pool: &database::Pool,
            id: Uuid,
        ) -> Result<Option<Self>, RunError> {
            database::query_many_to_many_single!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: super::Component::Inventory,
                },
                pool,
                DbInventoryItemRow,
                DbInventoryItemRows,
                Self,
                RunError,
                r#"SELECT
                    item.id AS id,
                    item.name AS name,
                    item.description AS description,
                    weight,
                    category.id AS category_id,
                    category.name AS category_name,
                    product.id AS "product_id?",
                    product.name AS "product_name?",
                    product.description AS "product_description?",
                    trip.name AS "trip_name?",
                    -- trip.date AS "trip_date?: TripDate",
                    trip.state AS "trip_state?: TripState"
                FROM inventory_items AS item
                INNER JOIN inventory_items_categories as category
                    ON item.category_id = category.id
                LEFT JOIN products AS product
                    ON item.product_id = product.id
                LEFT OUTER JOIN trip_items as ti
                    ON ti.item_id = item.id
                    AND ti.pick = TRUE
                LEFT OUTER JOIN trips as trip
                    ON ti.trip_id = trip.id
                WHERE
                    item.id = $1
                    AND item.user_id = $2"#,
                id,
                ctx.user.id,
            )
            .await
        }

        #[tracing::instrument]
        pub async fn name_exists(
            ctx: &Context,
            pool: &database::Pool,
            name: &str,
        ) -> Result<bool, RunError> {
            database::query_exists!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: super::Component::Inventory,
                },
                pool,
                "SELECT id
            FROM inventory_items
            WHERE
                name = $1
                AND user_id = $2",
                name,
                ctx.user.id
            )
            .await
        }

        #[tracing::instrument]
        pub async fn delete(
            ctx: &Context,
            pool: &database::Pool,
            id: Uuid,
        ) -> Result<bool, RunError> {
            let results = database::execute!(
                &database::QueryClassification {
                    query_type: database::QueryType::Delete,
                    component: super::Component::Inventory,
                },
                pool,
                RunError,
                "DELETE FROM inventory_items
            WHERE
                id = $1
                AND user_id = $2",
                id,
                ctx.user.id
            )
            .await?;

            Ok(results.rows_affected() != 0)
        }

        #[tracing::instrument]
        pub async fn update(
            ctx: &Context,
            pool: &database::Pool,
            id: Uuid,
            name: &str,
            weight: u32,
        ) -> Result<Uuid, RunError> {
            let weight = i32::try_from(weight).unwrap();
            database::execute_returning_uuid!(
                &database::QueryClassification {
                    query_type: database::QueryType::Update,
                    component: super::Component::Inventory,
                },
                pool,
                "UPDATE inventory_items AS item
            SET
                name = $1,
                weight = $2
            WHERE
                item.id = $3
                AND item.user_id = $4
            RETURNING item.category_id AS id
            ",
                name,
                weight,
                id,
                ctx.user.id
            )
            .await
        }

        #[tracing::instrument]
        pub async fn save(
            ctx: &Context,
            pool: &database::Pool,
            name: &str,
            category_id: Uuid,
            weight: u32,
        ) -> Result<Uuid, RunError> {
            let id = Uuid::new_v4();
            let weight = i32::try_from(weight).unwrap();

            database::execute!(
                &database::QueryClassification {
                    query_type: database::QueryType::Insert,
                    component: super::Component::Inventory,
                },
                pool,
                RunError,
                "INSERT INTO inventory_items
                (id, name, description, weight, category_id, user_id)
            VALUES
                ($1, $2, $3, $4, $5, $6)",
                id,
                name,
                "",
                weight,
                category_id,
                ctx.user.id
            )
            .await?;

            Ok(id)
        }

        #[tracing::instrument]
        pub async fn get_category_max_weight(
            ctx: &Context,
            pool: &database::Pool,
            category_id: Uuid,
        ) -> Result<i32, RunError> {
            let weight = database::execute_returning!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: super::Component::Inventory,
                },
                pool,
                RunError,
                "
                SELECT COALESCE(MAX(i_item.weight), 0) as weight
                FROM inventory_items_categories as category
                INNER JOIN inventory_items as i_item
                    ON i_item.category_id = category.id
                WHERE
                    category_id = $1
                    AND category.user_id = $2
            ",
                i32,
                |row| row.weight.unwrap(),
                category_id,
                ctx.user.id
            )
            .await?;

            Ok(weight)
        }
    }

    #[derive(Debug)]
    pub struct Item {
        pub id: Uuid,
        pub name: String,
        #[allow(dead_code)]
        pub description: Option<String>,
        pub weight: i32,
        pub category_id: Uuid,
    }

    pub struct DbInventoryItemsRow {
        pub id: Uuid,
        pub name: String,
        pub weight: i32,
        pub description: Option<String>,
        pub category_id: Uuid,
    }

    impl TryFrom<DbInventoryItemsRow> for Item {
        type Error = RunError;

        fn try_from(row: DbInventoryItemsRow) -> Result<Self, Self::Error> {
            Ok(Self {
                id: row.id,
                name: row.name,
                description: row.description, // TODO
                weight: row.weight,
                category_id: row.category_id,
            })
        }
    }

    impl Item {
        #[tracing::instrument(skip(pool))]
        pub async fn _get_category_total_picked_weight(
            ctx: &Context,
            pool: &database::Pool,
            category_id: Uuid,
        ) -> Result<i32, RunError> {
            database::execute_returning!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: super::Component::Inventory,
                },
                pool,
                RunError,
                "
                SELECT COALESCE(SUM(i_item.weight), 0) as weight
                FROM inventory_items_categories as category
                INNER JOIN inventory_items as i_item
                    ON i_item.category_id = category.id
                INNER JOIN trip_items as t_item
                    ON i_item.id = t_item.item_id
                WHERE
                    category_id = $1
                    AND category.user_id = $2
                    AND t_item.pick = true
            ",
                i32,
                |row| i32::try_from(row.weight.unwrap()).unwrap(),
                category_id,
                ctx.user.id,
            )
            .await
        }
    }
}

mod todo {
    use std::fmt;

    use axum::{
        body::Body,
        extract::{Form, Path, State as StateExtractor},
        http::HeaderMap,
        response::{IntoResponse, Redirect, Response},
        routing::post,
        Extension,
    };
    use leptos::prelude::*;
    use serde::Deserialize;
    use uuid::Uuid;

    use super::{error::RunError, user::User, AppState, Context, RequestError};

    use super::Trip;

    #[derive(Debug, PartialEq, Eq)]
    pub enum State {
        Todo,
        Done,
    }

    impl From<bool> for State {
        fn from(done: bool) -> Self {
            if done {
                Self::Done
            } else {
                Self::Todo
            }
        }
    }

    impl From<State> for bool {
        fn from(value: State) -> Self {
            match value {
                State::Todo => false,
                State::Done => true,
            }
        }
    }

    #[derive(Debug)]
    pub struct Todo {
        pub id: Id,
        pub description: String,
        pub state: State,
    }

    struct TodoRow {
        id: Uuid,
        description: String,
        done: bool,
    }

    impl TryFrom<TodoRow> for Todo {
        type Error = RunError;

        fn try_from(row: TodoRow) -> Result<Self, Self::Error> {
            Ok(Self {
                id: Id::new(row.id),
                description: row.description,
                state: row.done.into(),
            })
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Container {
        pub trip_id: Uuid,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Id(Uuid);

    impl std::fmt::Display for Id {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Id {
        #[must_use]
        pub fn new(id: Uuid) -> Self {
            Self(id)
        }
    }

    impl Todo {
        #[must_use]
        pub fn is_done(&self) -> bool {
            self.state == State::Done
        }
    }

    impl Todo {
        pub async fn findall(
            ctx: &Context,
            pool: &database::Pool,
            container: Container,
        ) -> Result<Vec<Self>, RunError> {
            let todos: Vec<Self> = database::query_all!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: super::Component::Todo,
                },
                pool,
                TodoRow,
                Todo,
                RunError,
                r"
                SELECT
                    todo.id AS id,
                    todo.description AS description,
                    todo.done AS done
                FROM trip_todos AS todo
                INNER JOIN trips
                    ON trips.id = todo.trip_id
                WHERE
                    trips.id = $1
                    AND trips.user_id = $2
            ",
                container.trip_id,
                ctx.user.id
            )
            .await?;

            Ok(todos)
        }

        #[tracing::instrument]
        async fn find(
            ctx: &Context,
            pool: &database::Pool,
            trip_id: Uuid,
            id: Uuid,
        ) -> Result<Option<Self>, RunError> {
            database::query_one!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: super::Component::Todo,
                },
                pool,
                TodoRow,
                Self,
                RunError,
                r"
                SELECT
                    todo.id AS id,
                    todo.description AS description,
                    todo.done AS done
                FROM trip_todos AS todo
                INNER JOIN trips
                    ON trips.id = todo.trip_id
                WHERE
                    trips.id = $1
                    AND todo.id = $2
                    AND trips.user_id = $3
            ",
                trip_id,
                id,
                ctx.user.id,
            )
            .await
        }
    }

    pub struct TodoNew {
        pub description: String,
    }

    #[derive(Debug)]
    #[cfg_attr(feature = "ssr", derive(sqlx::Type))]
    struct TodoId(uuid::Uuid);

    impl fmt::Display for TodoId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Todo {
        fn new_id() -> TodoId {
            TodoId(Uuid::new_v4())
        }

        async fn create(
            ctx: &Context,
            pool: &database::Pool,
            trip_id: Uuid,
            description: String,
        ) -> Result<TodoId, RunError> {
            let id = Self::new_id();
            tracing::info!("adding new todo with id {id}");
            database::execute!(
                &database::QueryClassification {
                    query_type: database::QueryType::Insert,
                    component: super::Component::Todo,
                },
                pool,
                RunError,
                r"
                INSERT INTO trip_todos
                    (id, description, done, trip_id)
                SELECT $1, $2, false, id as trip_id
                FROM trips
                WHERE id = $3 AND EXISTS(SELECT 1 FROM trips WHERE id = $3 and user_id = $4)
                LIMIT 1
            ",
                id.0,
                description,
                trip_id,
                ctx.user.id,
            )
            .await?;

            Ok(id)
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct StateUpdate {
        new_state: State,
    }

    impl From<bool> for StateUpdate {
        fn from(state: bool) -> Self {
            Self {
                new_state: state.into(),
            }
        }
    }

    impl From<State> for StateUpdate {
        fn from(new_state: State) -> Self {
            Self { new_state }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct DescriptionUpdate(String);

    impl From<String> for DescriptionUpdate {
        fn from(new_description: String) -> Self {
            Self(new_description)
        }
    }

    #[derive(Debug)]
    pub enum UpdateElement {
        State(StateUpdate),
        Description(DescriptionUpdate),
    }

    impl Todo {
        #[tracing::instrument]
        async fn update(
            ctx: &Context,
            pool: &database::Pool,
            trip_id: Uuid,
            id: TodoId,
            update_element: UpdateElement,
        ) -> Result<Option<Self>, RunError> {
            match update_element {
                UpdateElement::State(state) => {
                    let done = state == State::Done.into();

                    let result = database::query_one!(
                        &database::QueryClassification {
                            query_type: database::QueryType::Update,
                            component: super::Component::Trips,
                        },
                        pool,
                        TodoRow,
                        Todo,
                        RunError,
                        r"
                        UPDATE trip_todos
                            SET done = $1
                        WHERE trip_id = $2
                        AND id = $3
                        AND EXISTS(SELECT 1 FROM trips WHERE id = $2 AND user_id = $4)
                        RETURNING
                            id,
                            description,
                            done
                    ",
                        done,
                        trip_id,
                        id.0,
                        ctx.user.id
                    )
                    .await?;

                    Ok(result)
                }
                UpdateElement::Description(new_description) => {
                    let result = database::query_one!(
                        &database::QueryClassification {
                            query_type: database::QueryType::Update,
                            component: super::Component::Todo,
                        },
                        pool,
                        TodoRow,
                        Todo,
                        RunError,
                        r"
                        UPDATE trip_todos
                        SET description = $1
                        WHERE
                            id = $2
                            AND trip_id = $3
                            AND EXISTS(SELECT 1 FROM trips WHERE trip_id = $3 AND user_id = $4)
                        RETURNING
                            id,
                            description,
                            done
                    ",
                        new_description.0,
                        id.0,
                        trip_id,
                        ctx.user.id,
                    )
                    .await?;

                    Ok(result)
                }
            }
        }
    }

    impl Todo {
        #[tracing::instrument]
        async fn delete<'c, T>(
            ctx: &Context,
            db: T,
            trip_id: Uuid,
            id: Uuid,
        ) -> Result<bool, RunError>
        where
            T: sqlx::Acquire<'c, Database = sqlx::Postgres> + Send + std::fmt::Debug,
        {
            let results = database::execute!(
                &database::QueryClassification {
                    query_type: database::QueryType::Delete,
                    component: super::Component::Todo,
                },
                &mut *(db.acquire().await?),
                RunError,
                r"
                DELETE FROM trip_todos
                WHERE
                    id = $1
                    AND EXISTS (SELECT 1 FROM trips WHERE trip_id = $2 AND user_id = $3)
            ",
                id,
                trip_id,
                ctx.user.id,
            )
            .await?;

            Ok(results.rows_affected() != 0)
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum UiState {
        Default,
        Edit,
    }

    #[derive(Debug)]
    pub struct BuildInput {
        pub trip_id: Uuid,
        pub state: UiState,
    }

    impl Todo {
        #[tracing::instrument]
        fn build(&self) -> impl IntoView {
            todo!();
            view! {}
            // let done = self.is_done();
            // html!(
            //     li
            //         ."flex"
            //         ."flex-row"
            //         ."justify-start"
            //         ."items-stretch"
            //         ."bg-green-50"[done]
            //         ."bg-red-50"[!done]
            //         ."h-full"
            //     {
            //         @if input.state == UiState::Edit {
            //             form
            //                 name="edit-todo"
            //                 id="edit-todo"
            //                 action={
            //                     "/trips/" (input.trip_id)
            //                     "/todo/" (self.id)
            //                     "/edit/save"
            //                 }
            //                 target="_self"
            //                 method="post"
            //                 hx-post={
            //                     "/trips/" (input.trip_id)
            //                     "/todo/" (self.id)
            //                     "/edit/save"
            //                 }
            //                 hx-target="closest li"
            //                 hx-swap="outerHTML"
            //             {}
            //             div
            //                 ."flex"
            //                 ."flex-row"
            //                 ."aspect-square"
            //             {
            //                 span
            //                     ."mdi"
            //                     ."m-auto"
            //                     ."text-xl"
            //                     ."mdi-check"[self.is_done()]
            //                     ."mdi-checkbox-blank-outline"[!self.is_done()]
            //                 {}
            //             }
            //             div
            //                 ."p-2"
            //                 .grow
            //             {
            //                 input
            //                     ."w-full"
            //                     type="text"
            //                     form="edit-todo"
            //                     id="todo-description"
            //                     name="todo-description"
            //                     value=(self.description)
            //                 {}
            //             }
            //             button
            //                 type="submit"
            //                 form="edit-todo"
            //                 ."bg-green-200"
            //                 ."hover:bg-green-300"
            //                 ."flex"
            //                 ."flex-row"
            //                 ."aspect-square"
            //             {
            //                 span
            //                     ."mdi"
            //                     ."m-auto"
            //                     ."mdi-content-save"
            //                     ."text-xl"
            //                 {}
            //             }
            //             a
            //                 href="."
            //                 hx-post={
            //                     "/trips/" (input.trip_id)
            //                     "/todo/" (self.id)
            //                     "/edit/cancel"
            //                 }
            //                 hx-target="closest li"
            //                 hx-swap="outerHTML"
            //                 ."flex"
            //                 ."flex-row"
            //                 ."aspect-square"
            //                 ."bg-red-200"
            //                 ."hover:bg-red-300"
            //             {
            //                 span
            //                     ."mdi"
            //                     ."mdi-cancel"
            //                     ."text-xl"
            //                     ."m-auto"
            //                 {}
            //             }
            //         } @else {
            //             @if done {
            //                 a
            //                     ."flex"
            //                     ."flex-row"
            //                     ."aspect-square"
            //                     ."hover:bg-red-50"
            //                     href={
            //                         "/trips/" (input.trip_id)
            //                         "/todo/" (self.id)
            //                         "/done/false"
            //                     }
            //                     hx-post={
            //                         "/trips/" (input.trip_id)
            //                         "/todo/" (self.id)
            //                         "/done/htmx/false"
            //                     }
            //                     hx-target="closest li"
            //                     hx-swap="outerHTML"
            //                 {
            //                     span
            //                         ."mdi"
            //                         ."m-auto"
            //                         ."text-xl"
            //                         ."mdi-check"
            //                     {}
            //                 }
            //             } @else {
            //                 a
            //                     ."flex"
            //                     ."flex-row"
            //                     ."aspect-square"
            //                     ."hover:bg-green-50"
            //                     href={
            //                         "/trips/" (input.trip_id)
            //                         "/todo/" (self.id)
            //                         "/done/true"
            //                     }
            //                     hx-post={
            //                         "/trips/" (input.trip_id)
            //                         "/todo/" (self.id)
            //                         "/done/htmx/true"
            //                     }
            //                     hx-target="closest li"
            //                     hx-swap="outerHTML"
            //                 {
            //                     span
            //                         ."mdi"
            //                         ."m-auto"
            //                         ."text-xl"
            //                         ."mdi-checkbox-blank-outline"
            //                     {}
            //                 }
            //             }
            //             span
            //                 ."p-2"
            //                 ."grow"
            //             {
            //                 (self.description)
            //             }
            //             a
            //                 ."flex"
            //                 ."flex-row"
            //                 ."aspect-square"
            //                 ."bg-blue-200"
            //                 ."hover:bg-blue-400"
            //                 href=(format!("?edit_todo={id}", id = self.id))
            //                 hx-post={
            //                     "/trips/" (input.trip_id)
            //                     "/todo/" (self.id)
            //                     "/edit"
            //                 }
            //                 hx-target="closest li"
            //                 hx-swap="outerHTML"
            //             {
            //                 span ."m-auto" ."mdi" ."mdi-pencil" ."text-xl" {}
            //             }
            //             a
            //                 ."flex"
            //                 ."flex-row"
            //                 ."aspect-square"
            //                 ."bg-red-100"
            //                 ."hover:bg-red-200"
            //                 href=(format!("?delete_todo={id}", id = self.id))
            //                 hx-post={
            //                     "/trips/" (input.trip_id)
            //                     "/todo/" (self.id)
            //                     "/delete"
            //                 }
            //                 hx-target="#todolist"
            //                 hx-swap="outerHTML"
            //             {
            //                 span ."m-auto" ."mdi" ."mdi-delete-outline" ."text-xl" {}
            //             }
            //         }
            // }
            // )
        }
    }

    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct TripTodoNew {
        #[serde(rename = "new-todo-description")]
        description: String,
    }

    mod list {
        use uuid::Uuid;

        use leptos::prelude::*;

        use super::Todo;
        use super::Trip;

        #[derive(Debug)]
        pub struct List<'a> {
            pub trip: &'a Trip,
            pub todos: &'a Vec<Todo>,
        }

        #[derive(Debug)]
        pub struct BuildInput {
            pub edit_todo: Option<Uuid>,
        }

        impl List<'_> {
            #[tracing::instrument]
            fn build(&self) -> impl IntoView {
                todo!();
                view! {}
                // html!(
                //     div #todolist {
                //         h1 ."text-xl" ."mb-5" { "Todos" }
                //         ul
                //             ."flex"
                //             ."flex-col"
                //         {
                //             @for todo in self.todos {
                //                 @let state = input.edit_todo
                //                     .map_or(super::UiState::Default, |id| if todo.id == super::Id::new(id) {
                //                         super::UiState::Edit
                //                     } else {
                //                         super::UiState::Default
                //                     });
                //                 (todo.build(super::BuildInput{trip_id:self.trip.id, state}))
                //             }
                //             (NewTodo::build(&self.trip.id))
                //         }
                //     }
                // )
            }
        }

        pub struct NewTodo;

        impl NewTodo {
            #[tracing::instrument]
            pub fn build(trip_id: &Uuid) -> impl IntoView {
                todo!();
                view! {}
                // html!(
                //     li
                //         ."flex"
                //         ."flex-row"
                //         ."justify-start"
                //         ."items-stretch"
                //         ."h-full"
                //     {
                //         form
                //             name="new-todo"
                //             id="new-todo"
                //             action={
                //                 "/trips/" (trip_id)
                //                 "/todo/new"
                //             }
                //             target="_self"
                //             method="post"
                //             hx-post={
                //                 "/trips/" (trip_id)
                //                 "/todo/new"
                //             }
                //             hx-target="#todolist"
                //             hx-swap="outerHTML"
                //         {}
                //         button
                //             type="submit"
                //             form="new-todo"
                //             ."bg-green-200"
                //             ."hover:bg-green-300"
                //             ."flex"
                //             ."flex-row"
                //             ."aspect-square"
                //         {
                //             span
                //                 ."mdi"
                //                 ."m-auto"
                //                 ."mdi-plus"
                //                 ."text-xl"
                //             {}
                //         }
                //         div
                //             ."border-4"
                //             ."p-1"
                //             .grow
                //         {
                //             input
                //                 ."appearance-none"
                //                 ."w-full"
                //                 type="text"
                //                 form="new-todo"
                //                 id="new-todo-description"
                //                 name="new-todo-description"
                //             {}
                //         }
                //     }
                // )
            }
        }
    }
}

#[cfg(feature = "ssr")]
mod auth {
    use axum::{
        extract::{Request, State},
        middleware::Next,
        response::IntoResponse,
    };
    use futures::FutureExt;
    use tracing::Instrument;

    use super::user::User;

    use super::{AppState, AuthError, RunError};

    #[derive(Clone, Debug)]
    pub enum Config {
        Enabled,
        Disabled { assume_user: String },
    }

    #[tracing::instrument(name = "check_auth", skip(state, request, next))]
    pub async fn authorize(
        State(state): State<AppState>,
        mut request: Request,
        next: Next,
    ) -> Result<impl IntoResponse, RunError> {
        // We must not access `request` inside the async block above, otherwise there will be
        // errors like the following:
        //
        // the trait `tower::Service<http::Request<axum::body::Body>>` is not implemented for
        // `FromFn<fn(State<AppState>, Request<Body>, Next) -> impl Future<Output =
        // Result<impl IntoResponse, Error>> {authorize}, AppState, Route, _>
        //
        // I am honestly not sure about the reason
        let username_header = request.headers().get("x-auth-username");

        let user = async {
            let auth: Result<Result<User, AuthError>, RunError> = match state.auth_config {
                Config::Disabled { assume_user } => {
                    let user =
                        match super::user::User::find_by_name(&state.database_pool, &assume_user)
                            .await?
                        {
                            Some(user) => Ok(user),
                            None => Err(AuthError::AuthenticationUserNotFound {
                                username: assume_user,
                            }),
                        };
                    Ok(user)
                }
                Config::Enabled => match username_header {
                    None => Ok(Err(AuthError::AuthenticationHeaderMissing)),
                    Some(username) => match username.to_str() {
                        Err(e) => Ok(Err(AuthError::AuthenticationHeaderInvalid {
                            message: e.to_string(),
                        })),
                        Ok(username) => {
                            match super::user::User::find_by_name(&state.database_pool, username)
                                .await?
                            {
                                Some(user) => Ok(Ok(user)),
                                None => Ok(Err(AuthError::AuthenticationUserNotFound {
                                    username: username.to_string(),
                                })),
                            }
                        }
                    },
                },
            };
            auth
        }
        .instrument(tracing::debug_span!("authorize"))
        .inspect(|r| {
            if let Ok(auth) = r {
                match auth {
                    Ok(user) => tracing::debug!(?user, "auth successful"),
                    Err(e) => e.trace(),
                }
            }
        })
        .map(|r| {
            r.map(|auth| {
                metrics::counter!(
                    format!("packager_auth_{}_total", {
                        match auth {
                            Ok(_) => "success".to_string(),
                            Err(ref e) => {
                                format!("failure_{}", e.to_prom_metric_name())
                            }
                        }
                    }),
                    &match &auth {
                        Ok(user) => vec![("username", user.username.clone())],
                        Err(e) => e.to_prom_labels(),
                    }
                )
                .increment(1);
                auth
            })
        })
        // outer result: failure of the process, e.g. database connection failed
        // inner result: auth rejected, with AuthError
        .await??;

        request.extensions_mut().insert(user);
        Ok::<http::Response<axum::body::Body>, RunError>(next.run(request).await)
    }
}

mod errorpage {
    use leptos::prelude::*;

    pub struct ErrorPage;

    impl ErrorPage {
        #[tracing::instrument]
        pub fn build(message: &str) -> impl IntoView {
            todo!();
            view! {}
            // html!(
            //     (DOCTYPE)
            //     html {
            //         head {
            //             title { "Packager" }
            //         }
            //         body {
            //             h1 { "Error" }
            //             p { (message) }
            //         }
            //     }
            // )
        }
    }
}

#[cfg(feature = "ssr")]
mod telemetry {
    pub(super) mod metrics {
        use std::future::Future;

        use axum::routing::get;
        use axum::Router;

        use axum_prometheus::PrometheusMetricLayerBuilder;
        use tokio::net::TcpListener;

        use super::super::StartError;

        /// Serves metrics on the specified `addr`.
        ///
        /// You will get two outputs back: Another router, and a task that you have
        /// to run to actually spawn the metrics server endpoint
        pub fn prometheus_server(
            router: Router,
            addr: std::net::SocketAddr,
        ) -> (Router, impl Future<Output = Result<(), StartError>>) {
            let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
                .with_prefix(env!("CARGO_PKG_NAME"))
                .with_default_metrics()
                .build_pair();

            let app =
                Router::new().route("/metrics", get(|| async move { metric_handle.render() }));

            let task = async move {
                axum::serve(
                    TcpListener::bind(addr)
                        .await
                        .map_err(|e| StartError::Bind {
                            addr,
                            message: e.to_string(),
                        })?,
                    app,
                )
                .await
                // Error = Infallible
                .unwrap();
                unreachable!()
            };

            (router.layer(prometheus_layer), task)
        }
    }
}
