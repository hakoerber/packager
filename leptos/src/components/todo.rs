use std::fmt;

use leptos::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use super::Component;
use crate::{context::Context, error::RunError};

use super::trip::Trip;

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
                component: Component::Todo,
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
                component: Component::Todo,
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
                component: Component::Todo,
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
                        component: Component::Trip,
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
                        component: Component::Todo,
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
    async fn delete<'c, T>(ctx: &Context, db: T, trip_id: Uuid, id: Uuid) -> Result<bool, RunError>
    where
        T: sqlx::Acquire<'c, Database = sqlx::Postgres> + Send + std::fmt::Debug,
    {
        let results = database::execute!(
            &database::QueryClassification {
                query_type: database::QueryType::Delete,
                component: Component::Todo,
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
