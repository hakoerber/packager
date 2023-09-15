use maud::{html, Markup};
use uuid::Uuid;

use crate::components::crud;
use crate::components::view::{self, *};

use async_trait::async_trait;

use crate::{models::Error, sqlite, Context};

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

#[derive(Debug)]
pub struct TodoFilter {
    pub trip_id: Uuid,
}

impl Todo {
    pub fn is_done(&self) -> bool {
        self.state == State::Done
    }
}

#[async_trait]
impl crud::Read for Todo {
    type Filter = TodoFilter;
    type Id = Uuid;

    async fn findall(
        ctx: &Context,
        pool: &sqlite::Pool,
        filter: TodoFilter,
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
        filter: TodoFilter,
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
    type Filter = TodoFilter;
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
pub enum TodoUpdate {
    State(State),
    Description(String),
}

#[async_trait]
impl crud::Update for Todo {
    type Id = Uuid;
    type Filter = TodoFilter;
    type Update = TodoUpdate;

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
            TodoUpdate::State(state) => {
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
            TodoUpdate::Description(new_description) => {
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
impl<'c> crud::Delete<'c> for Todo {
    type Id = Uuid;
    type Filter = TodoFilter;

    #[tracing::instrument]
    async fn delete<T>(ctx: &Context, db: T, filter: TodoFilter, id: Uuid) -> Result<bool, Error>
    where
        T: sqlx::Acquire<'c, Database = sqlx::Sqlite> + Send + std::fmt::Debug,
    {
        use sqlx::Acquire;

        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();
        let trip_id_param = filter.trip_id.to_string();

        let mut transaction = db.begin().await?;
        let conn = transaction.acquire().await?;

        let results = crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Delete,
                component: sqlite::Component::Todo,
            },
            conn,
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
pub enum TodoUiState {
    Default,
    Edit,
}

#[derive(Debug)]
pub struct TodoBuildInput {
    pub trip_id: Uuid,
    pub state: TodoUiState,
}

impl view::View for Todo {
    type Input = TodoBuildInput;

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
                @if input.state == TodoUiState::Edit {
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

pub struct NewTodo;

impl NewTodo {
    #[tracing::instrument]
    pub fn build(trip_id: &Uuid) -> Markup {
        html!(
            li
                ."flex"
                ."flex-row"
                ."justify-start"
                ."items-stretch"
                ."h-full"
            {
                form
                    name="new-todo"
                    id="new-todo"
                    action={
                        "/trips/" (trip_id)
                        "/todo/new"
                    }
                    target="_self"
                    method="post"
                    hx-post={
                        "/trips/" (trip_id)
                        "/todo/new"
                    }
                    hx-target="#todolist"
                    hx-swap="outerHTML"
                {}
                button
                    type="submit"
                    form="new-todo"
                    ."bg-green-200"
                    ."hover:bg-green-300"
                    ."flex"
                    ."flex-row"
                    ."aspect-square"
                {
                    span
                        ."mdi"
                        ."m-auto"
                        ."mdi-plus"
                        ."text-xl"
                    {}
                }
                div
                    ."border-4"
                    ."p-1"
                    .grow
                {
                    input
                        ."appearance-none"
                        ."w-full"
                        type="text"
                        form="new-todo"
                        id="new-todo-description"
                        name="new-todo-description"
                    {}
                }
            }
        )
    }
}

#[derive(Debug)]
pub struct TodoList<'a> {
    pub trip: &'a Trip,
    pub todos: &'a Vec<Todo>,
}

impl<'a> TodoList<'a> {
    #[tracing::instrument]
    pub fn build(&self, edit_todo: Option<Uuid>) -> Markup {
        html!(
            div #todolist {
                h1 ."text-xl" ."mb-5" { "Todos" }
                ul
                    ."flex"
                    ."flex-col"
                {
                    @for todo in self.todos {
                        @let state = edit_todo
                            .map(|id| if todo.id == id {
                                TodoUiState::Edit
                            } else {
                                TodoUiState::Default
                            }).unwrap_or(TodoUiState::Default);
                        (todo.build(TodoBuildInput{trip_id:self.trip.id, state}))
                    }
                    (NewTodo::build(&self.trip.id))
                }
            }
        )
    }
}
