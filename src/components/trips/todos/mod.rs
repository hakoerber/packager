#![allow(unused_variables)]

pub mod list;
pub use list::List;

use axum::{
    body::{BoxBody, HttpBody},
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
    components::{
        self,
        crud::{self, Read, Update},
        route::{self, Toggle},
        view::{self, View},
    },
    htmx,
    models::{user::User, Error},
    routing::get_referer,
    sqlite, AppState, Context, RequestError,
};

use async_trait::async_trait;

use crate::models::trips::Trip;

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
    pub id: Uuid,
    pub description: String,
    pub state: State,
}

struct TodoRow {
    id: String,
    description: String,
    done: bool,
}

impl TryFrom<TodoRow> for Todo {
    type Error = Error;

    fn try_from(row: TodoRow) -> Result<Self, Self::Error> {
        Ok(Todo {
            id: Uuid::try_parse(&row.id)?,
            description: row.description,
            state: row.done.into(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub trip_id: Uuid,
}

impl From<(Uuid, Uuid)> for Filter {
    fn from((trip_id, _todo_id): (Uuid, Uuid)) -> Self {
        Self { trip_id }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Id(Uuid);

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Id {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<(Uuid, Uuid)> for Id {
    fn from((_trip_id, todo_id): (Uuid, Uuid)) -> Self {
        Self(todo_id)
    }
}

impl Todo {
    pub fn is_done(&self) -> bool {
        self.state == State::Done
    }
}

#[async_trait]
impl crud::Read for Todo {
    type Filter = Filter;
    type Id = Id;

    async fn findall(
        ctx: &Context,
        pool: &sqlite::Pool,
        filter: Filter,
    ) -> Result<Vec<Self>, Error> {
        let trip_id_param = filter.trip_id.to_string();
        let user_id = ctx.user.id.to_string();

        let todos: Vec<Todo> = crate::query_all!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Todo,
            },
            pool,
            TodoRow,
            Todo,
            r#"
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
            "#,
            trip_id_param,
            user_id,
        )
        .await?;

        Ok(todos)
    }

    #[tracing::instrument]
    async fn find(
        ctx: &Context,
        pool: &sqlite::Pool,
        filter: Filter,
        todo_id: Id,
    ) -> Result<Option<Self>, Error> {
        let trip_id_param = filter.trip_id.to_string();
        let todo_id_param = todo_id.0.to_string();
        let user_id = ctx.user.id.to_string();
        crate::query_one!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Todo,
            },
            pool,
            TodoRow,
            Self,
            r#"
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
            "#,
            trip_id_param,
            todo_id_param,
            user_id,
        )
        .await
    }
}

pub struct TodoNew {
    pub description: String,
}

#[async_trait]
impl crud::Create for Todo {
    type Id = Id;
    type Filter = Filter;
    type Info = TodoNew;

    async fn create(
        ctx: &Context,
        pool: &sqlite::Pool,
        filter: Self::Filter,
        info: Self::Info,
    ) -> Result<Self::Id, Error> {
        let user_id = ctx.user.id.to_string();
        let id = Uuid::new_v4();
        tracing::info!("adding new todo with id {id}");
        let id_param = id.to_string();
        let trip_id_param = filter.trip_id.to_string();
        crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Insert,
                component: sqlite::Component::Todo,
            },
            pool,
            r#"
                INSERT INTO trip_todos
                    (id, description, done, trip_id)
                SELECT ?, ?, false, id as trip_id
                FROM trips
                WHERE trip_id = ? AND EXISTS(SELECT 1 FROM trips WHERE id = ? and user_id = ?)
                LIMIT 1
            "#,
            id_param,
            info.description,
            trip_id_param,
            trip_id_param,
            user_id,
        )
        .await?;

        Ok(components::trips::todos::Id(id))
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

#[async_trait]
impl crud::Update for Todo {
    type Id = Id;
    type Filter = Filter;
    type UpdateElement = UpdateElement;

    #[tracing::instrument]
    async fn update(
        ctx: &Context,
        pool: &sqlite::Pool,
        filter: Self::Filter,
        id: Self::Id,
        update_element: Self::UpdateElement,
    ) -> Result<Option<Self>, Error> {
        let user_id = ctx.user.id.to_string();
        let trip_id_param = filter.trip_id.to_string();
        let todo_id_param = id.to_string();
        match update_element {
            UpdateElement::State(state) => {
                let done = state == State::Done.into();

                let result = crate::query_one!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Trips,
                    },
                    pool,
                    TodoRow,
                    Todo,
                    r#"
                        UPDATE trip_todos
                            SET done = ?
                        WHERE trip_id = ?
                        AND id = ?
                        AND EXISTS(SELECT 1 FROM trips WHERE id = ? AND user_id = ?)
                        RETURNING
                            id,
                            description,
                            done
                    "#,
                    done,
                    trip_id_param,
                    todo_id_param,
                    trip_id_param,
                    user_id
                )
                .await?;

                Ok(result)
            }
            UpdateElement::Description(new_description) => {
                let user_id = ctx.user.id.to_string();
                let trip_id_param = filter.trip_id.to_string();
                let todo_id_param = id.to_string();

                let result = crate::query_one!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Todo,
                    },
                    pool,
                    TodoRow,
                    Todo,
                    r#"
                        UPDATE trip_todos
                        SET description = ?
                        WHERE 
                            id = ? 
                            AND trip_id = ?
                            AND EXISTS(SELECT 1 FROM trips WHERE trip_id = ? AND user_id = ?)
                        RETURNING
                            id,
                            description,
                            done
                    "#,
                    new_description.0,
                    todo_id_param,
                    trip_id_param,
                    trip_id_param,
                    user_id,
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
    type Filter = Filter;

    #[tracing::instrument]
    async fn delete<'c, T>(ctx: &Context, db: T, filter: &Filter, id: Id) -> Result<bool, Error>
    where
        T: sqlx::Acquire<'c, Database = sqlx::Sqlite> + Send + std::fmt::Debug,
    {
        let id_param = id.0.to_string();
        let user_id = ctx.user.id.to_string();
        let trip_id_param = filter.trip_id.to_string();

        let results = crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Delete,
                component: sqlite::Component::Todo,
            },
            &mut *(db.acquire().await?),
            r#"
                DELETE FROM trip_todos
                WHERE
                    id = ?
                    AND EXISTS (SELECT 1 FROM trips WHERE trip_id = ? AND user_id = ?)
            "#,
            id_param,
            trip_id_param,
            user_id,
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
pub struct TripTodoNew {
    #[serde(rename = "new-todo-description")]
    description: String,
}

#[async_trait]
impl route::Create for Todo {
    type Form = TripTodoNew;
    type UrlParams = (Uuid,);

    const URL: &'static str = "/:id/todo/new";

    #[tracing::instrument]
    async fn create(
        Extension(current_user): Extension<User>,
        StateExtractor(state): StateExtractor<AppState>,
        headers: HeaderMap,
        Path((trip_id,)): Path<Self::UrlParams>,
        Form(form): Form<Self::Form>,
    ) -> Result<Response<BoxBody>, crate::Error> {
        let ctx = Context::build(current_user);
        // method output is not required as we reload the whole trip todos anyway
        let _todo_item = <Self as crud::Create>::create(
            &ctx,
            &state.database_pool,
            Filter { trip_id },
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

    const URL: &'static str = "/:id/todo/:id/delete";

    #[tracing::instrument]
    async fn delete(
        Extension(current_user): Extension<User>,
        StateExtractor(state): StateExtractor<AppState>,
        _headers: HeaderMap,
        Path((trip_id, todo_id)): Path<Self::UrlParams>,
    ) -> Result<Response<BoxBody>, crate::Error> {
        let ctx = Context::build(current_user);
        let deleted = <Self as crud::Delete>::delete(
            &ctx,
            &state.database_pool,
            &Filter { trip_id },
            components::trips::todos::Id(todo_id),
        )
        .await?;

        if !deleted {
            return Err(crate::Error::Request(RequestError::NotFound {
                message: format!("todo with id {todo_id} not found"),
            }));
        }

        let trip = crate::models::trips::Trip::find(&ctx, &state.database_pool, trip_id).await?;
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
    fn router<B>() -> axum::Router<AppState, B>
    where
        B: HttpBody + Send + 'static,
        <B as HttpBody>::Data: Send,
        <B as HttpBody>::Error: std::error::Error + Sync + Send,
    {
        axum::Router::new()
            .route("/new", axum::routing::post(<Self as route::Create>::create))
            .route(
                "/:id/delete",
                axum::routing::post(<Self as route::Delete>::delete),
            )
            .merge(StateUpdate::router())
    }
}

#[tracing::instrument]
pub async fn trip_todo_done(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    Todo::update(
        &ctx,
        &state.database_pool,
        Filter { trip_id },
        Id(todo_id),
        UpdateElement::State(State::Done.into()),
    )
    .await?;

    Ok(Redirect::to(get_referer(&headers)?))
}

#[tracing::instrument]
pub async fn trip_todo_undone_htmx(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    Todo::update(
        &ctx,
        &state.database_pool,
        Filter { trip_id },
        Id(todo_id),
        UpdateElement::State(State::Todo.into()),
    )
    .await?;

    let todo_item = Todo::find(&ctx, &state.database_pool, Filter { trip_id }, Id(todo_id))
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
pub async fn trip_todo_undone(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    Todo::update(
        &ctx,
        &state.database_pool,
        Filter { trip_id },
        Id(todo_id),
        UpdateElement::State(State::Todo.into()),
    )
    .await?;

    Ok(Redirect::to(get_referer(&headers)?))
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TripTodoDescription {
    #[serde(rename = "todo-description")]
    description: String,
}

#[tracing::instrument]
pub async fn trip_todo_edit(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    headers: HeaderMap,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    let todo_item = Todo::find(&ctx, &state.database_pool, Filter { trip_id }, Id(todo_id)).await?;

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
pub async fn trip_todo_edit_save(
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
        Filter { trip_id },
        Id(todo_id),
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
pub async fn trip_todo_edit_cancel(
    Extension(current_user): Extension<User>,
    StateExtractor(state): StateExtractor<AppState>,
    headers: HeaderMap,
    Path((trip_id, todo_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, crate::Error> {
    let ctx = Context::build(current_user);
    let todo_item = Todo::find(&ctx, &state.database_pool, Filter { trip_id }, Id(todo_id)).await?;

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
    type Id = Id;
    type Filter = Filter;

    async fn set(
        ctx: &Context,
        pool: &sqlite::Pool,
        filter: Self::Filter,
        id: Self::Id,
        value: bool,
    ) -> Result<(), crate::Error> {
        Todo::update(&ctx, &pool, filter, id, UpdateElement::State(value.into())).await?;
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
    ) -> Result<Response<BoxBody>, crate::Error> {
        let ctx = Context::build(current_user);
        <Self as crud::Toggle>::set(
            &ctx,
            &state.database_pool,
            Filter { trip_id },
            Id(todo_id),
            value,
        )
        .await?;

        Ok(Redirect::to(get_referer(&headers)?).into_response())
    }

    fn router<B>() -> axum::Router<AppState, B>
    where
        B: HttpBody + Send + 'static,
        <B as HttpBody>::Data: Send,
        <B as HttpBody>::Error: std::error::Error + Sync + Send,
    {
        axum::Router::new()
            .route(Self::URL_TRUE, post(Self::set_true))
            .route(Self::URL_FALSE, post(Self::set_false))
    }
}

#[async_trait]
impl route::ToggleHtmx for StateUpdate {
    type Id = Id;
    type Filter = Filter;
    type UrlParams = (Uuid, Uuid);

    const URL_TRUE: &'static str = "/:id/done/htmx/true";
    const URL_FALSE: &'static str = "/:id/done/htmx/false";

    async fn set(
        current_user: User,
        state: AppState,
        params: Self::UrlParams,
        value: bool,
    ) -> Result<(crate::Context, AppState, Self::UrlParams, bool), crate::Error> {
        let ctx = Context::build(current_user);
        <Self as crud::Toggle>::set(
            &ctx,
            &state.database_pool,
            params.into(),
            params.into(),
            value,
        )
        .await?;

        Ok((ctx, state, params, value))
    }

    async fn response(
        ctx: &Context,
        state: AppState,
        (trip_id, todo_id): Self::UrlParams,
        value: bool,
    ) -> Result<Response<BoxBody>, crate::Error> {
        let todo_item = Todo::find(&ctx, &state.database_pool, Filter { trip_id }, Id(todo_id))
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

    fn router<B>() -> axum::Router<AppState, B>
    where
        B: HttpBody + Send + 'static,
        <B as HttpBody>::Data: Send,
        <B as HttpBody>::Error: std::error::Error + Sync + Send,
    {
        axum::Router::new()
            .route(Self::URL_TRUE, post(Self::on))
            .route(Self::URL_FALSE, post(Self::off))
    }
}

#[async_trait]
impl route::Toggle for StateUpdate {}
