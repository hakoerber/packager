pub mod list;
pub use list::List;

use axum::{
    body::BoxBody,
    extract::{Form, Path},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use maud::{html, Markup};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    components::{
        crud, route,
        view::{self, View},
    },
    htmx,
    models::Error,
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

impl Todo {
    pub fn is_done(&self) -> bool {
        self.state == State::Done
    }
}

#[async_trait]
impl crud::Read for Todo {
    type Filter = Filter;
    type Id = Uuid;

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
        todo_id: Uuid,
    ) -> Result<Option<Self>, Error> {
        let trip_id_param = filter.trip_id.to_string();
        let todo_id_param = todo_id.to_string();
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
    type Id = Uuid;
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

        Ok(id)
    }
}

#[derive(Debug)]
pub enum Update {
    State(State),
    Description(String),
}

#[async_trait]
impl crud::Update for Todo {
    type Id = Uuid;
    type Filter = Filter;
    type Update = Update;

    #[tracing::instrument]
    async fn update(
        ctx: &Context,
        pool: &sqlite::Pool,
        filter: Self::Filter,
        id: Self::Id,
        update: Self::Update,
    ) -> Result<Option<Self>, Error> {
        let user_id = ctx.user.id.to_string();
        let trip_id_param = filter.trip_id.to_string();
        let todo_id_param = id.to_string();
        match update {
            Update::State(state) => {
                let done = state == State::Done;

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
            Update::Description(new_description) => {
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
                    new_description,
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
    type Id = Uuid;
    type Filter = Filter;

    #[tracing::instrument]
    async fn delete<'c, T>(ctx: &Context, db: T, filter: &Filter, id: Uuid) -> Result<bool, Error>
    where
        T: sqlx::Acquire<'c, Database = sqlx::Sqlite> + Send + std::fmt::Debug,
    {
        let id_param = id.to_string();
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
                                "/undone"
                            }
                            hx-post={
                                "/trips/" (input.trip_id)
                                "/todo/" (self.id)
                                "/undone"
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
                                "/done"
                            }
                            hx-post={
                                "/trips/" (input.trip_id)
                                "/todo/" (self.id)
                                "/done"
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
    type ParentUrlParams = (Uuid,);
    type UrlParams = ();

    const URL: &'static str = "/new";

    #[tracing::instrument]
    async fn create(
        Extension(current_user): Extension<crate::models::user::User>,
        axum::extract::State(state): axum::extract::State<AppState>,
        headers: HeaderMap,
        Path(((trip_id,), ())): Path<(Self::ParentUrlParams, Self::UrlParams)>,
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
    type ParentUrlParams = (Uuid,);
    type UrlParams = (Uuid,);

    const URL: &'static str = "/:id/delete";

    #[tracing::instrument]
    async fn delete(
        Extension(current_user): Extension<crate::models::user::User>,
        axum::extract::State(state): axum::extract::State<AppState>,
        _headers: HeaderMap,
        Path(((trip_id,), (todo_id,))): Path<(Self::ParentUrlParams, Self::UrlParams)>,
    ) -> Result<Response<BoxBody>, crate::Error> {
        let ctx = Context::build(current_user);
        let deleted = <Self as crud::Delete>::delete(
            &ctx,
            &state.database_pool,
            &Filter { trip_id },
            todo_id,
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
