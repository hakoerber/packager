pub(crate) mod list;
pub(crate) use list::List;

use axum::{
    body::Body,
    extract::{Form, Path, State as StateExtractor},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    routing::post,
    Extension,
};
use maud::{html, Markup};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    domains::{
        self,
        crud::{self, Read, Update},
        route::{self, Toggle},
        view::{self, View},
    },
    db,
    error::Error,
    htmx,
    models::User,
    routing::get_referer,
    AppState, Context, RequestError,
};

use async_trait::async_trait;

use super::model::Trip;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum State {
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
pub(crate) struct Todo {
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
    type Error = Error;

    fn try_from(row: TodoRow) -> Result<Self, Self::Error> {
        Ok(Todo {
            id: Id::new(row.id),
            description: row.description,
            state: row.done.into(),
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct Container {
    pub trip_id: Uuid,
}

impl crud::Container for Container {
    type Id = Id;
    type Reference = Reference;

    fn with_id(&self, id: Self::Id) -> Self::Reference {
        Reference {
            id,
            container: *self,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Reference {
    pub id: Id,
    pub container: Container,
}

impl From<(Uuid, Uuid)> for Reference {
    fn from((trip_id, todo_id): (Uuid, Uuid)) -> Self {
        Self {
            id: Id::new(todo_id),
            container: Container { trip_id },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Id(Uuid);

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

#[async_trait]
impl crud::Read for Todo {
    type Reference = Reference;
    type Container = Container;

    async fn findall(
        ctx: &Context,
        pool: &db::Pool,
        container: Container,
    ) -> Result<Vec<Self>, Error> {
        let todos: Vec<Todo> = crate::query_all!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Todo,
            },
            pool,
            TodoRow,
            Todo,
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
        pool: &db::Pool,
        reference: Reference,
    ) -> Result<Option<Self>, Error> {
        crate::query_one!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Todo,
            },
            pool,
            TodoRow,
            Self,
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
            reference.container.trip_id,
            reference.id.0,
            ctx.user.id,
        )
        .await
    }
}

pub(crate) struct TodoNew {
    pub description: String,
}

#[async_trait]
impl crud::Create for Todo {
    type Id = Id;
    type Container = Container;
    type Info = TodoNew;

    fn new_id() -> Self::Id {
        Id::new(Uuid::new_v4())
    }

    async fn create(
        ctx: &Context,
        pool: &db::Pool,
        container: Self::Container,
        info: Self::Info,
    ) -> Result<Self::Id, Error> {
        let id = Self::new_id();
        tracing::info!("adding new todo with id {id}");
        crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Insert,
                component: db::Component::Todo,
            },
            pool,
            r"
                INSERT INTO trip_todos
                    (id, description, done, trip_id)
                SELECT $1, $2, false, id as trip_id
                FROM trips
                WHERE id = $3 AND EXISTS(SELECT 1 FROM trips WHERE id = $3 and user_id = $4)
                LIMIT 1
            ",
            id.0,
            info.description,
            container.trip_id,
            ctx.user.id,
        )
        .await?;

        Ok(id)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct StateUpdate {
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
pub(crate) struct DescriptionUpdate(String);

impl From<String> for DescriptionUpdate {
    fn from(new_description: String) -> Self {
        Self(new_description)
    }
}

#[derive(Debug)]
pub(crate) enum UpdateElement {
    State(StateUpdate),
    Description(DescriptionUpdate),
}

#[async_trait]
impl crud::Update for Todo {
    type Reference = Reference;
    type UpdateElement = UpdateElement;

    #[tracing::instrument]
    async fn update(
        ctx: &Context,
        pool: &db::Pool,
        reference: Self::Reference,
        update_element: Self::UpdateElement,
    ) -> Result<Option<Self>, Error> {
        match update_element {
            UpdateElement::State(state) => {
                let done = state == State::Done.into();

                let result = crate::query_one!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Trips,
                    },
                    pool,
                    TodoRow,
                    Todo,
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
                    reference.container.trip_id,
                    reference.id.0,
                    ctx.user.id
                )
                .await?;

                Ok(result)
            }
            UpdateElement::Description(new_description) => {
                let result = crate::query_one!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Todo,
                    },
                    pool,
                    TodoRow,
                    Todo,
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
                    reference.id.0,
                    reference.container.trip_id,
                    ctx.user.id,
                )
                .await?;

                Ok(result)
            }
        }
    }
}

#[async_trait]
impl crud::Delete for Todo {
    type Id = Id;
    type Container = Container;
    type Reference = Reference;

    #[tracing::instrument]
    async fn delete<'c, T>(ctx: &Context, db: T, reference: &Reference) -> Result<bool, Error>
    where
        T: sqlx::Acquire<'c, Database = sqlx::Postgres> + Send + std::fmt::Debug,
    {
        let results = crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Delete,
                component: db::Component::Todo,
            },
            &mut *(db.acquire().await?),
            r"
                DELETE FROM trip_todos
                WHERE
                    id = $1
                    AND EXISTS (SELECT 1 FROM trips WHERE trip_id = $2 AND user_id = $3)
            ",
            reference.id.0,
            reference.container.trip_id,
            ctx.user.id,
        )
        .await?;

        Ok(results.rows_affected() != 0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum UiState {
    Default,
    Edit,
}

#[derive(Debug)]
pub(crate) struct BuildInput {
    pub trip_id: Uuid,
    pub state: UiState,
}

impl view::View for Todo {
    type Input = BuildInput;

    #[tracing::instrument]
    fn build(&self, input: Self::Input) -> Markup {
        let done = self.is_done();
        html!(
            li
                ."flex"
                ."flex-row"
                ."justify-start"
                ."items-stretch"
                ."bg-green-50"[done]
                ."bg-red-50"[!done]
                ."h-full"
            {
                @if input.state == UiState::Edit {
                    form
                        name="edit-todo"
                        id="edit-todo"
                        action={
                            "/trips/" (input.trip_id)
                            "/todo/" (self.id)
                            "/edit/save"
                        }
                        target="_self"
                        method="post"
                        hx-post={
                            "/trips/" (input.trip_id)
                            "/todo/" (self.id)
                            "/edit/save"
                        }
                        hx-target="closest li"
                        hx-swap="outerHTML"
                    {}
                    div
                        ."flex"
                        ."flex-row"
                        ."aspect-square"
                    {
                        span
                            ."mdi"
                            ."m-auto"
                            ."text-xl"
                            ."mdi-check"[self.is_done()]
                            ."mdi-checkbox-blank-outline"[!self.is_done()]
                        {}
                    }
                    div
                        ."p-2"
                        .grow
                    {
                        input
                            ."w-full"
                            type="text"
                            form="edit-todo"
                            id="todo-description"
                            name="todo-description"
                            value=(self.description)
                        {}
                    }
                    button
                        type="submit"
                        form="edit-todo"
                        ."bg-green-200"
                        ."hover:bg-green-300"
                        ."flex"
                        ."flex-row"
                        ."aspect-square"
                    {
                        span
                            ."mdi"
                            ."m-auto"
                            ."mdi-content-save"
                            ."text-xl"
                        {}
                    }
                    a
                        href="."
                        hx-post={
                            "/trips/" (input.trip_id)
                            "/todo/" (self.id)
                            "/edit/cancel"
                        }
                        hx-target="closest li"
                        hx-swap="outerHTML"
                        ."flex"
                        ."flex-row"
                        ."aspect-square"
                        ."bg-red-200"
                        ."hover:bg-red-300"
                    {
                        span
                            ."mdi"
                            ."mdi-cancel"
                            ."text-xl"
                            ."m-auto"
                        {}
                    }
                } @else {
                    @if done {
                        a
                            ."flex"
                            ."flex-row"
                            ."aspect-square"
                            ."hover:bg-red-50"
                            href={
                                "/trips/" (input.trip_id)
                                "/todo/" (self.id)
                                "/done/false"
                            }
                            hx-post={
                                "/trips/" (input.trip_id)
                                "/todo/" (self.id)
                                "/done/htmx/false"
                            }
                            hx-target="closest li"
                            hx-swap="outerHTML"
                        {
                            span
                                ."mdi"
                                ."m-auto"
                                ."text-xl"
                                ."mdi-check"
                            {}
                        }
                    } @else {
                        a
                            ."flex"
                            ."flex-row"
                            ."aspect-square"
                            ."hover:bg-green-50"
                            href={
                                "/trips/" (input.trip_id)
                                "/todo/" (self.id)
                                "/done/true"
                            }
                            hx-post={
                                "/trips/" (input.trip_id)
                                "/todo/" (self.id)
                                "/done/htmx/true"
                            }
                            hx-target="closest li"
                            hx-swap="outerHTML"
                        {
                            span
                                ."mdi"
                                ."m-auto"
                                ."text-xl"
                                ."mdi-checkbox-blank-outline"
                            {}
                        }
                    }
                    span
                        ."p-2"
                        ."grow"
                    {
                        (self.description)
                    }
                    a
                        ."flex"
                        ."flex-row"
                        ."aspect-square"
                        ."bg-blue-200"
                        ."hover:bg-blue-400"
                        href=(format!("?edit_todo={id}", id = self.id))
                        hx-post={
                            "/trips/" (input.trip_id)
                            "/todo/" (self.id)
                            "/edit"
                        }
                        hx-target="closest li"
                        hx-swap="outerHTML"
                    {
                        span ."m-auto" ."mdi" ."mdi-pencil" ."text-xl" {}
                    }
                    a
                        ."flex"
                        ."flex-row"
                        ."aspect-square"
                        ."bg-red-100"
                        ."hover:bg-red-200"
                        href=(format!("?delete_todo={id}", id = self.id))
                        hx-post={
                            "/trips/" (input.trip_id)
                            "/todo/" (self.id)
                            "/delete"
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

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct TripTodoNew {
    #[serde(rename = "new-todo-description")]
    description: String,
}

#[async_trait]
impl route::Create for Todo {
    type Form = TripTodoNew;
    type UrlParams = (Uuid,);

    // const URL: &'static str = "/:id/todo/new";

    #[tracing::instrument]
    async fn create(
        Extension(current_user): Extension<User>,
        StateExtractor(state): StateExtractor<AppState>,
        headers: HeaderMap,
        Path((trip_id,)): Path<Self::UrlParams>,
        Form(form): Form<Self::Form>,
    ) -> Result<Response<Body>, crate::Error> {
        let ctx = Context::build(current_user);
        // method output is not required as we reload the whole trip todos anyway
        let _todo_item = <Self as crud::Create>::create(
            &ctx,
            &state.database_pool,
            Container { trip_id },
            TodoNew {
                description: form.description,
            },
        )
        .await?;

        if htmx::is_htmx(&headers) {
            let trip = Trip::find(&ctx, &state.database_pool, trip_id).await?;
            match trip {
                None => Err(crate::Error::Request(RequestError::NotFound {
                    message: format!("trip with id {trip_id} not found"),
                })),
                Some(mut trip) => {
                    trip.load_todos(&ctx, &state.database_pool).await?;
                    Ok(list::List {
                        trip: &trip,
                        todos: trip.todos(),
                    }
                    .build(list::BuildInput { edit_todo: None })
                    .into_response())
                }
            }
        } else {
            Ok(Redirect::to(&format!("/trips/{trip_id}/")).into_response())
        }
    }
}

#[async_trait]
impl route::Delete for Todo {
    type UrlParams = (Uuid, Uuid);

    // const URL: &'static str = "/:id/todo/:id/delete";

    #[tracing::instrument]
    async fn delete(
        Extension(current_user): Extension<User>,
        StateExtractor(state): StateExtractor<AppState>,
        _headers: HeaderMap,
        Path((trip_id, todo_id)): Path<Self::UrlParams>,
    ) -> Result<Response<Body>, crate::Error> {
        let ctx = Context::build(current_user);
        let deleted = <Self as crud::Delete>::delete(
            &ctx,
            &state.database_pool,
            &Reference {
                container: Container { trip_id },
                id: domains::trips::todos::Id(todo_id),
            },
        )
        .await?;

        if !deleted {
            return Err(crate::Error::Request(RequestError::NotFound {
                message: format!("todo with id {todo_id} not found"),
            }));
        }

        let trip = super::model::Trip::find(&ctx, &state.database_pool, trip_id).await?;
        match trip {
            None => Err(crate::Error::Request(RequestError::NotFound {
                message: format!("trip with id {trip_id} not found"),
            })),
            Some(mut trip) => {
                trip.load_todos(&ctx, &state.database_pool).await?;
                Ok(list::List {
                    trip: &trip,
                    todos: trip.todos(),
                }
                .build(list::BuildInput { edit_todo: None })
                .into_response())
            }
        }
    }
}

impl route::Router for Todo {
    fn router() -> axum::Router<AppState> {
        axum::Router::new()
            .route("/:id/edit", post(edit_todo))
            .route("/:id/edit/save", post(edit_todo_save))
            .route("/:id/edit/cancel", post(edit_todo_cancel))
            .route("/new", axum::routing::post(<Self as route::Create>::create))
            .route(
                "/:id/delete",
                axum::routing::post(<Self as route::Delete>::delete),
            )
            .merge(StateUpdate::router())
    }
}

#[tracing::instrument]
#[allow(dead_code)]
pub(crate) async fn trip_todo_done(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    Todo::update(
        &ctx,
        &state.database_pool,
        Reference {
            id: Id(todo_id),
            container: Container { trip_id },
        },
        UpdateElement::State(State::Done.into()),
    )
    .await?;

    Ok(Redirect::to(get_referer(&headers)?))
}

#[tracing::instrument]
#[allow(dead_code)]
pub(crate) async fn trip_todo_undone_htmx(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    Todo::update(
        &ctx,
        &state.database_pool,
        Reference {
            id: Id(todo_id),
            container: Container { trip_id },
        },
        UpdateElement::State(State::Todo.into()),
    )
    .await?;

    let todo_item = Todo::find(
        &ctx,
        &state.database_pool,
        Reference {
            id: Id(todo_id),
            container: Container { trip_id },
        },
    )
    .await?
    .ok_or_else(|| {
        crate::Error::Request(RequestError::NotFound {
            message: format!("todo with id {todo_id} not found"),
        })
    })?;

    Ok(todo_item.build(BuildInput {
        trip_id,
        state: UiState::Default,
    }))
}

#[tracing::instrument]
#[allow(dead_code)]
pub(crate) async fn trip_todo_undone(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    Todo::update(
        &ctx,
        &state.database_pool,
        Reference {
            id: Id(todo_id),
            container: Container { trip_id },
        },
        UpdateElement::State(State::Todo.into()),
    )
    .await?;

    Ok(Redirect::to(get_referer(&headers)?))
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct TripTodoDescription {
    #[serde(rename = "todo-description")]
    description: String,
}

#[tracing::instrument]
pub(crate) async fn edit_todo(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    headers: HeaderMap,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    let todo_item = Todo::find(
        &ctx,
        &state.database_pool,
        Reference {
            id: Id(todo_id),
            container: Container { trip_id },
        },
    )
    .await?;

    match todo_item {
        None => Err(crate::Error::Request(RequestError::NotFound {
            message: format!("todo with id {todo_id} not found"),
        })),
        Some(todo_item) => Ok(todo_item
            .build(BuildInput {
                trip_id,
                state: UiState::Edit,
            })
            .into_response()),
    }
}

#[tracing::instrument]
pub(crate) async fn edit_todo_save(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    headers: HeaderMap,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
    Form(form): Form<TripTodoDescription>,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    let todo_item = Todo::update(
        &ctx,
        &state.database_pool,
        Reference {
            id: Id(todo_id),
            container: Container { trip_id },
        },
        UpdateElement::Description(form.description.into()),
    )
    .await?;

    match todo_item {
        None => Err(crate::Error::Request(RequestError::NotFound {
            message: format!("todo with id {todo_id} not found"),
        })),
        Some(todo_item) => {
            if htmx::is_htmx(&headers) {
                Ok(todo_item
                    .build(BuildInput {
                        trip_id,
                        state: UiState::Default,
                    })
                    .into_response())
            } else {
                Ok(Redirect::to(&format!("/trips/{trip_id}/")).into_response())
            }
        }
    }
}

#[tracing::instrument]
pub(crate) async fn edit_todo_cancel(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    headers: HeaderMap,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    let todo_item = Todo::find(
        &ctx,
        &state.database_pool,
        Reference {
            id: Id(todo_id),
            container: Container { trip_id },
        },
    )
    .await?;

    match todo_item {
        None => Err(crate::Error::Request(RequestError::NotFound {
            message: format!("todo with id {todo_id} not found"),
        })),
        Some(todo_item) => Ok(todo_item
            .build(BuildInput {
                trip_id,
                state: UiState::Default,
            })
            .into_response()),
    }
}

#[async_trait]
impl crud::Toggle for StateUpdate {
    type Reference = Reference;

    async fn set(
        ctx: &Context,
        pool: &db::Pool,
        reference: Self::Reference,
        value: bool,
    ) -> Result<(), crate::Error> {
        Todo::update(ctx, pool, reference, UpdateElement::State(value.into())).await?;
        Ok(())
    }
}

#[async_trait]
impl route::ToggleFallback for StateUpdate {
    type UrlParams = (Uuid, Uuid);

    const URL_TRUE: &'static str = "/:id/done/true";
    const URL_FALSE: &'static str = "/:id/done/false";

    async fn set(
        current_user: User,
        state: AppState,
        headers: HeaderMap,
        (trip_id, todo_id): (Uuid, Uuid),
        value: bool,
    ) -> Result<Response<Body>, crate::Error> {
        let ctx = Context::build(current_user);
        <Self as crud::Toggle>::set(
            &ctx,
            &state.database_pool,
            Reference {
                id: Id(todo_id),
                container: Container { trip_id },
            },
            value,
        )
        .await?;

        Ok(Redirect::to(get_referer(&headers)?).into_response())
    }

    fn router() -> axum::Router<AppState> {
        axum::Router::new()
            .route(Self::URL_TRUE, post(Self::set_true))
            .route(Self::URL_FALSE, post(Self::set_false))
    }
}

#[async_trait]
impl route::ToggleHtmx for StateUpdate {
    type UrlParams = (Uuid, Uuid);

    const URL_TRUE: &'static str = "/:id/done/htmx/true";
    const URL_FALSE: &'static str = "/:id/done/htmx/false";

    async fn set(
        current_user: User,
        state: AppState,
        params: Self::UrlParams,
        value: bool,
    ) -> Result<(crate::Context, AppState, Self::UrlParams), crate::Error> {
        let ctx = Context::build(current_user);
        <Self as crud::Toggle>::set(&ctx, &state.database_pool, params.into(), value).await?;

        Ok((ctx, state, params))
    }

    async fn response(
        ctx: &Context,
        state: AppState,
        (trip_id, todo_id): Self::UrlParams,
    ) -> Result<Response<Body>, crate::Error> {
        let todo_item = Todo::find(
            ctx,
            &state.database_pool,
            Reference {
                id: Id(todo_id),
                container: Container { trip_id },
            },
        )
        .await?
        .ok_or_else(|| {
            crate::Error::Request(RequestError::NotFound {
                message: format!("todo with id {todo_id} not found"),
            })
        })?;

        Ok(todo_item
            .build(BuildInput {
                trip_id,
                state: UiState::Default,
            })
            .into_response())
    }

    fn router() -> axum::Router<AppState> {
        axum::Router::new()
            .route(Self::URL_TRUE, post(Self::on))
            .route(Self::URL_FALSE, post(Self::off))
    }
}

#[async_trait]
impl route::Toggle for StateUpdate {}
