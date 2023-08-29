use axum::{
    extract::{Extension, Path, Query, State},
    http::header::{self, HeaderMap, HeaderName, HeaderValue},
    middleware::{self, Next},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};

use hyper::Request;

use serde::Deserialize;

use uuid::Uuid;

use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

mod html;
mod view;

type User = models::user::User;

use error::{Error, RequestError, StartError};

#[derive(Clone)]
pub enum AuthConfig {
    Enabled,
    Disabled { assume_user: String },
}

#[derive(Clone)]
pub struct AppState {
    database_pool: sqlite::Pool<sqlite::Sqlite>,
    client_state: ClientState,
    auth_config: AuthConfig,
}

#[derive(Clone)]
pub struct Context {
    user: User,
}

impl Context {
    fn build(user: User) -> Self {
        Self { user }
    }
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    database_url: String,
    #[arg(long, default_value_t = 3000)]
    port: u16,
    #[arg(long)]
    bind: String,
    #[arg(long, name = "USERNAME")]
    disable_auth_and_assume_user: Option<String>,
}

#[derive(Clone)]
pub struct ClientState {
    pub active_category_id: Option<Uuid>,
    pub edit_item: Option<Uuid>,
    pub trip_edit_attribute: Option<models::trips::TripAttribute>,
    pub trip_type_edit: Option<Uuid>,
}

impl ClientState {
    pub fn new() -> Self {
        ClientState {
            active_category_id: None,
            edit_item: None,
            trip_edit_attribute: None,
            trip_type_edit: None,
        }
    }
}

impl Default for ClientState {
    fn default() -> Self {
        Self::new()
    }
}

struct UriPath(String);

impl fmt::Display for UriPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> Into<&'a str> for &'a UriPath {
    fn into(self) -> &'a str {
        self.0.as_str()
    }
}

#[derive(PartialEq, Eq)]
pub enum TopLevelPage {
    Inventory,
    Trips,
}

impl TopLevelPage {
    fn id(&self) -> &'static str {
        match self {
            Self::Inventory => "inventory",
            Self::Trips => "trips",
        }
    }

    fn path(&self) -> UriPath {
        UriPath(
            match self {
                Self::Inventory => "/inventory/",
                Self::Trips => "/trips/",
            }
            .to_string(),
        )
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Inventory => "Inventory",
            Self::Trips => "Trips",
        }
    }
}

enum HtmxEvents {
    TripItemEdited,
}

impl From<HtmxEvents> for HeaderValue {
    fn from(val: HtmxEvents) -> Self {
        HeaderValue::from_static(val.to_str())
    }
}

impl HtmxEvents {
    fn to_str(&self) -> &'static str {
        match self {
            Self::TripItemEdited => "TripItemEdited",
        }
    }
}

enum HtmxResponseHeaders {
    Trigger,
    PushUrl,
}

impl From<HtmxResponseHeaders> for HeaderName {
    fn from(val: HtmxResponseHeaders) -> Self {
        match val {
            HtmxResponseHeaders::Trigger => HeaderName::from_static("hx-trigger"),
            HtmxResponseHeaders::PushUrl => HeaderName::from_static("hx-push-url"),
        }
    }
}

enum HtmxRequestHeaders {
    HtmxRequest,
}

impl From<HtmxRequestHeaders> for HeaderName {
    fn from(val: HtmxRequestHeaders) -> Self {
        match val {
            HtmxRequestHeaders::HtmxRequest => HeaderName::from_static("hx-request"),
        }
    }
}

async fn authorize<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, Error> {
    let current_user = match state.auth_config {
        AuthConfig::Disabled { assume_user } => {
            match models::user::User::find_by_name(&state.database_pool, &assume_user).await? {
                Some(user) => user,
                None => {
                    return Err(Error::Request(RequestError::AuthenticationUserNotFound {
                        username: assume_user,
                    }))
                }
            }
        }
        AuthConfig::Enabled => {
            let Some(username) = request.headers().get("x-auth-username") else {
                return Err(Error::Request(RequestError::AuthenticationHeaderMissing));
            };

            let username = username
                .to_str()
                .map_err(|error| {
                    Error::Request(RequestError::AuthenticationHeaderInvalid {
                        message: error.to_string(),
                    })
                })?
                .to_string();

            match models::user::User::find_by_name(&state.database_pool, &username).await? {
                Some(user) => user,
                None => {
                    return Err(Error::Request(RequestError::AuthenticationUserNotFound {
                        username,
                    }))
                }
            }
        }
    };

    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() -> Result<(), StartError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();

    let database_pool = sqlie::init_database_pool(&args.database_url).await?;
    sqlite::migrate(&database_pool).await?;

    let state = AppState {
        database_pool,
        client_state: ClientState::new(),
        auth_config: if let Some(assume_user) = args.disable_auth_and_assume_user {
            AuthConfig::Disabled { assume_user }
        } else {
            AuthConfig::Enabled
        },
    };

    let icon_handler = || async {
        (
            [(header::CONTENT_TYPE, "image/svg+xml")],
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/luggage.svg")),
        )
    };

    // build our application with a route
    let app = Router::new()
        .route("/favicon.svg", get(icon_handler))
        .route("/assets/luggage.svg", get(icon_handler))
        .route(
            "/notfound",
            get(|| async {
                Error::Request(RequestError::NotFound {
                    message: "hi".to_string(),
                })
            }),
        )
        .route("/debug", get(debug))
        .merge(
            // thse are routes that require authentication
            Router::new()
                .route("/", get(root))
                .nest(
                    (&TopLevelPage::Trips.path()).into(),
                    Router::new()
                        .route("/", get(trips).post(trip_create))
                        .route("/types/", get(trips_types).post(trip_type_create))
                        .route("/types/:id/edit/name/submit", post(trips_types_edit_name))
                        .route("/:id/", get(trip))
                        .route("/:id/comment/submit", post(trip_comment_set))
                        .route("/:id/categories/:id/select", post(trip_category_select))
                        .route("/:id/packagelist/", get(trip_packagelist))
                        .route(
                            "/:id/packagelist/item/:id/pack",
                            post(trip_item_packagelist_set_pack_htmx),
                        )
                        .route(
                            "/:id/packagelist/item/:id/unpack",
                            post(trip_item_packagelist_set_unpack_htmx),
                        )
                        .route(
                            "/:id/packagelist/item/:id/ready",
                            post(trip_item_packagelist_set_ready_htmx),
                        )
                        .route(
                            "/:id/packagelist/item/:id/unready",
                            post(trip_item_packagelist_set_unready_htmx),
                        )
                        .route("/:id/state/:id", post(trip_state_set))
                        .route("/:id/total_weight", get(trip_total_weight_htmx))
                        .route("/:id/type/:id/add", get(trip_type_add))
                        .route("/:id/type/:id/remove", get(trip_type_remove))
                        .route("/:id/edit/:attribute/submit", post(trip_edit_attribute))
                        .route(
                            "/:id/items/:id/pick",
                            get(trip_item_set_pick).post(trip_item_set_pick_htmx),
                        )
                        .route(
                            "/:id/items/:id/unpick",
                            get(trip_item_set_unpick).post(trip_item_set_unpick_htmx),
                        )
                        .route(
                            "/:id/items/:id/pack",
                            get(trip_item_set_pack).post(trip_item_set_pack_htmx),
                        )
                        .route(
                            "/:id/items/:id/unpack",
                            get(trip_item_set_unpack).post(trip_item_set_unpack_htmx),
                        )
                        .route(
                            "/:id/items/:id/ready",
                            get(trip_item_set_ready).post(trip_item_set_ready_htmx),
                        )
                        .route(
                            "/:id/items/:id/unready",
                            get(trip_item_set_unready).post(trip_item_set_unready_htmx),
                        ),
                )
                .nest(
                    (&TopLevelPage::Inventory.path()).into(),
                    Router::new()
                        .route("/", get(inventory_inactive))
                        .route("/categories/:id/select", post(inventory_category_select))
                        .route("/category/", post(inventory_category_create))
                        .route("/category/:id/", get(inventory_active))
                        .route("/item/", post(inventory_item_create))
                        .route("/item/:id/", get(inventory_item))
                        .route("/item/:id/cancel", get(inventory_item_cancel))
                        .route("/item/:id/delete", get(inventory_item_delete))
                        .route("/item/:id/edit", post(inventory_item_edit))
                        .route("/item/name/validate", post(inventory_item_validate_name)),
                )
                .layer(middleware::from_fn_with_state(state.clone(), authorize)),
        )
        .fallback(|| async {
            Error::Request(RequestError::NotFound {
                message: "no route found".to_string(),
            })
        })
        .with_state(state);

    let addr = SocketAddr::from((
        IpAddr::from_str(&args.bind)
            .map_err(|error| format!("error parsing bind address {}: {}", &args.bind, error))
            .unwrap(),
        args.port,
    ));
    tracing::debug!("listening on {}", addr);
    axum::Server::try_bind(&addr)
        .map_err(|error| format!("error binding to {}: {}", addr, error))
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn root(Extension(current_user): Extension<User>) -> impl IntoResponse {
    view::Root::build(
        &Context::build(current_user),
        &view::home::Home::build(),
        None,
    )
}

async fn debug(headers: HeaderMap) -> impl IntoResponse {
    let out = {
        let mut out = String::new();
        for (key, value) in headers.iter() {
            out.push_str(&format!("{}: {}\n", key, value.to_str().unwrap()));
        }
        out
    };
    out
}

#[derive(Deserialize, Default)]
struct InventoryQuery {
    edit_item: Option<Uuid>,
}

async fn inventory_active(
    Extension(current_user): Extension<User>,
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(inventory_query): Query<InventoryQuery>,
) -> Result<impl IntoResponse, Error> {
    state.client_state.edit_item = inventory_query.edit_item;
    state.client_state.active_category_id = Some(id);

    let inventory = models::inventory::Inventory::load(&state.database_pool).await?;

    let active_category: Option<&models::inventory::Category> = state
        .client_state
        .active_category_id
        .map(|id| {
            inventory
                .categories
                .iter()
                .find(|category| category.id == id)
                .ok_or(Error::Request(RequestError::NotFound {
                    message: format!("a category with id {id} does not exist"),
                }))
        })
        .transpose()?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::inventory::Inventory::build(
            active_category,
            &inventory.categories,
            state.client_state.edit_item,
        ),
        Some(&TopLevelPage::Inventory),
    ))
}

async fn inventory_inactive(
    Extension(current_user): Extension<User>,
    State(mut state): State<AppState>,
    Query(inventory_query): Query<InventoryQuery>,
) -> Result<impl IntoResponse, Error> {
    state.client_state.edit_item = inventory_query.edit_item;
    state.client_state.active_category_id = None;

    let inventory = models::inventory::Inventory::load(&state.database_pool).await?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::inventory::Inventory::build(
            None,
            &inventory.categories,
            state.client_state.edit_item,
        ),
        Some(&TopLevelPage::Inventory),
    ))
}

#[derive(Deserialize)]
struct NewItem {
    #[serde(rename = "new-item-name")]
    name: String,
    #[serde(rename = "new-item-weight")]
    weight: u32,
    // damn i just love how serde is integrated everywhere, just add a feature to the uuid in
    // cargo.toml and go
    #[serde(rename = "new-item-category-id")]
    category_id: Uuid,
}

#[derive(Deserialize)]
struct NewItemName {
    #[serde(rename = "new-item-name")]
    name: String,
}

async fn inventory_item_validate_name(
    State(state): State<AppState>,
    Form(new_item): Form<NewItemName>,
) -> Result<impl IntoResponse, Error> {
    let exists =
        models::inventory::InventoryItem::name_exists(&state.database_pool, &new_item.name).await?;

    Ok(view::inventory::InventoryNewItemFormName::build(
        Some(&new_item.name),
        exists,
    ))
}

async fn inventory_item_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(new_item): Form<NewItem>,
) -> Result<impl IntoResponse, Error> {
    if new_item.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id = models::inventory::InventoryItem::save(
        &state.database_pool,
        &new_item.name,
        new_item.category_id,
        new_item.weight,
    )
    .await?;

    if is_htmx(&headers) {
        let inventory = models::inventory::Inventory::load(&state.database_pool).await?;

        // it's impossible to NOT find the item here, as we literally just added
        // it.
        let active_category: Option<&models::inventory::Category> = Some(
            inventory
                .categories
                .iter()
                .find(|category| category.id == new_item.category_id)
                .unwrap(),
        );

        Ok(view::inventory::Inventory::build(
            active_category,
            &inventory.categories,
            state.client_state.edit_item,
        )
        .into_response())
    } else {
        Ok(Redirect::to(&format!(
            "/inventory/category/{id}/",
            id = new_item.category_id
        ))
        .into_response())
    }
}

fn get_referer<'a>(headers: &'a HeaderMap) -> Result<&'a str, Error> {
    headers
        .get("referer")
        .ok_or(Error::Request(RequestError::RefererNotFound))?
        .to_str()
        .map_err(|error| {
            Error::Request(RequestError::RefererInvalid {
                message: error.to_string(),
            })
        })
}

async fn inventory_item_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Redirect, Error> {
    let deleted = models::inventory::InventoryItem::delete(&state.database_pool, id).await?;

    if !deleted {
        Err(Error::Request(RequestError::NotFound {
            message: format!("item with id {id} not found"),
        }))
    } else {
        Ok(Redirect::to(get_referer(&headers)?))
    }
}

#[derive(Deserialize)]
struct EditItem {
    #[serde(rename = "edit-item-name")]
    name: String,
    #[serde(rename = "edit-item-weight")]
    weight: u32,
}

async fn inventory_item_edit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(edit_item): Form<EditItem>,
) -> Result<Redirect, Error> {
    if edit_item.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let id = models::inventory::InventoryItem::update(
        &state.database_pool,
        id,
        &edit_item.name,
        edit_item.weight,
    )
    .await?;

    Ok(Redirect::to(&format!("/inventory/category/{id}/", id = id)))
}

async fn inventory_item_cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Redirect, Error> {
    let id = models::inventory::InventoryItem::find(&state.database_pool, id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("item with id {id} not found"),
        }))?;

    Ok(Redirect::to(&format!(
        "/inventory/category/{id}/",
        id = id.category.id
    )))
}

#[derive(Deserialize)]
struct NewTrip {
    #[serde(rename = "new-trip-name")]
    name: String,
    #[serde(rename = "new-trip-start-date")]
    date_start: time::Date,
    #[serde(rename = "new-trip-end-date")]
    date_end: time::Date,
}

async fn trip_create(
    State(state): State<AppState>,
    Form(new_trip): Form<NewTrip>,
) -> Result<Redirect, Error> {
    if new_trip.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let new_id = models::trips::Trip::save(
        &state.database_pool,
        &new_trip.name,
        new_trip.date_start,
        new_trip.date_end,
    )
    .await?;

    Ok(Redirect::to(&format!("/trips/{new_id}/")))
}

async fn trips(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    let trips = models::trips::Trip::all(&state.database_pool).await?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::trip::TripManager::build(trips),
        Some(&TopLevelPage::Trips),
    ))
}

#[derive(Debug, Deserialize)]
struct TripQuery {
    edit: Option<models::trips::TripAttribute>,
    category: Option<Uuid>,
}

async fn trip(
    Extension(current_user): Extension<User>,
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(trip_query): Query<TripQuery>,
) -> Result<impl IntoResponse, Error> {
    state.client_state.trip_edit_attribute = trip_query.edit;
    state.client_state.active_category_id = trip_query.category;

    let mut trip: models::trips::Trip = models::trips::Trip::find(&state.database_pool, id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("trip with id {id} not found"),
        }))?;

    trip.load_trips_types(&state.database_pool).await?;

    trip.sync_trip_items_with_inventory(&state.database_pool)
        .await?;

    trip.load_categories(&state.database_pool).await?;

    let active_category: Option<&models::trips::TripCategory> = state
        .client_state
        .active_category_id
        .map(|id| {
            trip.categories()
                .iter()
                .find(|category| category.category.id == id)
                .ok_or(Error::Request(RequestError::NotFound {
                    message: format!("an active category with id {id} does not exist"),
                }))
        })
        .transpose()?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::trip::Trip::build(
            &trip,
            state.client_state.trip_edit_attribute,
            active_category,
        ),
        Some(&TopLevelPage::Trips),
    ))
}

async fn trip_type_remove(
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, Error> {
    let found =
        models::trips::Trip::trip_type_remove(&state.database_pool, trip_id, type_id).await?;

    if !found {
        Err(Error::Request(RequestError::NotFound {
            message: format!("type {type_id} is not active for trip {trip_id}"),
        }))
    } else {
        Ok(Redirect::to(&format!("/trips/{trip_id}/")))
    }
}

async fn trip_type_add(
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, Error> {
    models::trips::Trip::trip_type_add(&state.database_pool, trip_id, type_id).await?;

    Ok(Redirect::to(&format!("/trips/{trip_id}/")))
}

#[derive(Deserialize)]
struct CommentUpdate {
    #[serde(rename = "new-comment")]
    new_comment: String,
}

async fn trip_comment_set(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
    Form(comment_update): Form<CommentUpdate>,
) -> Result<Redirect, Error> {
    let found = models::trips::Trip::set_comment(
        &state.database_pool,
        trip_id,
        &comment_update.new_comment,
    )
    .await?;

    if !found {
        Err(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))
    } else {
        Ok(Redirect::to(&format!("/trips/{id}/", id = trip_id)))
    }
}

#[derive(Deserialize)]
struct TripUpdate {
    #[serde(rename = "new-value")]
    new_value: String,
}

async fn trip_edit_attribute(
    State(state): State<AppState>,
    Path((trip_id, attribute)): Path<(Uuid, models::trips::TripAttribute)>,
    Form(trip_update): Form<TripUpdate>,
) -> Result<Redirect, Error> {
    if attribute == models::trips::TripAttribute::Name {
        if trip_update.new_value.is_empty() {
            return Err(Error::Request(RequestError::EmptyFormElement {
                name: "name".to_string(),
            }));
        }
    }
    models::trips::Trip::set_attribute(
        &state.database_pool,
        trip_id,
        attribute,
        &trip_update.new_value,
    )
    .await?;

    Ok(Redirect::to(&format!("/trips/{trip_id}/")))
}

async fn trip_item_set_state(
    state: &AppState,
    trip_id: Uuid,
    item_id: Uuid,
    key: models::trips::TripItemStateKey,
    value: bool,
) -> Result<(), Error> {
    models::trips::TripItem::set_state(&state.database_pool, trip_id, item_id, key, value).await?;
    Ok(())
}

async fn trip_row(
    state: &AppState,
    trip_id: Uuid,
    item_id: Uuid,
) -> Result<impl IntoResponse, Error> {
    let item = models::trips::TripItem::find(&state.database_pool, trip_id, item_id)
        .await?
        .ok_or_else(|| {
            Error::Request(RequestError::NotFound {
                message: format!("item with id {item_id} not found for trip {trip_id}"),
            })
        })?;

    let item_row = view::trip::TripItemListRow::build(
        trip_id,
        &item,
        models::inventory::InventoryItem::get_category_max_weight(
            &state.database_pool,
            item.item.category_id,
        )
        .await?,
    );

    let category =
        models::trips::TripCategory::find(&state.database_pool, trip_id, item.item.category_id)
            .await?
            .ok_or_else(|| {
                Error::Request(RequestError::NotFound {
                    message: format!("category with id {} not found", item.item.category_id),
                })
            })?;

    // TODO biggest_category_weight?
    let category_row = view::trip::TripCategoryListRow::build(trip_id, &category, true, 0, true);

    Ok(html::concat(item_row, category_row))
}

async fn trip_item_set_pick(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Pick,
            true,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

async fn trip_item_set_pick_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pick,
        true,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

async fn trip_item_set_unpick(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Pick,
            false,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

async fn trip_item_set_unpick_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pick,
        false,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

async fn trip_item_set_pack(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Pack,
            true,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

async fn trip_item_set_pack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
        true,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

async fn trip_item_set_unpack(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Pack,
            false,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

async fn trip_item_set_unpack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
        false,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

async fn trip_item_set_ready(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Ready,
            true,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

async fn trip_item_set_ready_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
        true,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

async fn trip_item_set_unready(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Ready,
            false,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

async fn trip_item_set_unready_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
        false,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

async fn trip_total_weight_htmx(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let total_weight =
        models::trips::Trip::find_total_picked_weight(&state.database_pool, trip_id).await?;
    Ok(view::trip::TripInfoTotalWeightRow::build(
        trip_id,
        total_weight,
    ))
}

#[derive(Deserialize)]
struct NewCategory {
    #[serde(rename = "new-category-name")]
    name: String,
}

async fn inventory_category_create(
    State(state): State<AppState>,
    Form(new_category): Form<NewCategory>,
) -> Result<Redirect, Error> {
    if new_category.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id =
        models::inventory::Category::save(&state.database_pool, &new_category.name).await?;

    Ok(Redirect::to("/inventory/"))
}

async fn trip_state_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((trip_id, new_state)): Path<(Uuid, models::trips::TripState)>,
) -> Result<impl IntoResponse, Error> {
    let exists = models::trips::Trip::set_state(&state.database_pool, trip_id, &new_state).await?;

    if !exists {
        return Err(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }));
    }

    if is_htmx(&headers) {
        Ok(view::trip::TripInfoStateRow::build(&new_state).into_response())
    } else {
        Ok(Redirect::to(&format!("/trips/{id}/", id = trip_id)).into_response())
    }
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get::<HeaderName>(HtmxRequestHeaders::HtmxRequest.into())
        .map(|value| value == "true")
        .unwrap_or(false)
}

#[derive(Debug, Deserialize)]
struct TripTypeQuery {
    edit: Option<Uuid>,
}

async fn trips_types(
    Extension(current_user): Extension<User>,
    State(mut state): State<AppState>,
    Query(trip_type_query): Query<TripTypeQuery>,
) -> Result<impl IntoResponse, Error> {
    state.client_state.trip_type_edit = trip_type_query.edit;

    let trip_types: Vec<models::trips::TripsType> =
        models::trips::TripsType::all(&state.database_pool).await?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::trip::types::TypeList::build(&state.client_state, trip_types),
        Some(&TopLevelPage::Trips),
    ))
}

#[derive(Deserialize)]
struct NewTripType {
    #[serde(rename = "new-trip-type-name")]
    name: String,
}

async fn trip_type_create(
    State(state): State<AppState>,
    Form(new_trip_type): Form<NewTripType>,
) -> Result<Redirect, Error> {
    if new_trip_type.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id = models::trips::TripsType::save(&state.database_pool, &new_trip_type.name).await?;

    Ok(Redirect::to("/trips/types/"))
}

#[derive(Deserialize)]
struct TripTypeUpdate {
    #[serde(rename = "new-value")]
    new_value: String,
}

async fn trips_types_edit_name(
    State(state): State<AppState>,
    Path(trip_type_id): Path<Uuid>,
    Form(trip_update): Form<TripTypeUpdate>,
) -> Result<Redirect, Error> {
    if trip_update.new_value.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let exists = models::trips::TripsType::set_name(
        &state.database_pool,
        trip_type_id,
        &trip_update.new_value,
    )
    .await?;

    if !exists {
        return Err(Error::Request(RequestError::NotFound {
            message: format!("trip type with id {trip_type_id} not found"),
        }));
    } else {
        Ok(Redirect::to("/trips/types/"))
    }
}

async fn inventory_item(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let item = models::inventory::InventoryItem::find(&state.database_pool, id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("inventory item with id {id} not found"),
        }))?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::inventory::InventoryItem::build(&state.client_state, &item),
        Some(&TopLevelPage::Inventory),
    ))
}

async fn trip_category_select(
    State(state): State<AppState>,
    Path((trip_id, category_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let mut trip = models::trips::Trip::find(&state.database_pool, trip_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))?;

    trip.load_categories(&state.database_pool).await?;

    let active_category = trip
        .categories()
        .iter()
        .find(|c| c.category.id == category_id)
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("category with id {category_id} not found"),
        }))?;

    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::PushUrl.into(),
        format!("?={category_id}").parse().unwrap(),
    );

    Ok((
        headers,
        view::trip::TripItems::build(Some(active_category), &trip),
    ))
}

async fn inventory_category_select(
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let inventory = models::inventory::Inventory::load(&state.database_pool).await?;

    let active_category: Option<&models::inventory::Category> = Some(
        inventory
            .categories
            .iter()
            .find(|category| category.id == category_id)
            .ok_or(Error::Request(RequestError::NotFound {
                message: format!("a category with id {category_id} not found"),
            }))?,
    );

    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::PushUrl.into(),
        format!("/inventory/category/{category_id}/")
            .parse()
            .unwrap(),
    );

    Ok((
        headers,
        view::inventory::Inventory::build(
            active_category,
            &inventory.categories,
            state.client_state.edit_item,
        ),
    ))
}

async fn trip_packagelist(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let mut trip = models::trips::Trip::find(&state.database_pool, trip_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))?;

    trip.load_categories(&state.database_pool).await?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::trip::packagelist::TripPackageList::build(&trip),
        Some(&TopLevelPage::Trips),
    ))
}

async fn trip_item_packagelist_set_pack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
        true,
    )
    .await?;

    let item = models::trips::TripItem::find(&state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowReady::build(
        trip_id, &item,
    ))
}

async fn trip_item_packagelist_set_unpack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
        false,
    )
    .await?;

    // note that this cannot fail due to a missing item, as trip_item_set_state would already
    // return 404. but error handling cannot hurt ;)
    let item = models::trips::TripItem::find(&state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowReady::build(
        trip_id, &item,
    ))
}

async fn trip_item_packagelist_set_ready_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
        true,
    )
    .await?;

    let item = models::trips::TripItem::find(&state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowUnready::build(
        trip_id, &item,
    ))
}

async fn trip_item_packagelist_set_unready_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
        false,
    )
    .await?;

    // note that this cannot fail due to a missing item, as trip_item_set_state would already
    // return 404. but error handling cannot hurt ;)
    let item = models::trips::TripItem::find(&state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowUnready::build(
        trip_id, &item,
    ))
}
