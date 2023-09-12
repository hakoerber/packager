use uuid::Uuid;

use crate::{
    models::{Error, QueryError},
    sqlite, Context,
};

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

impl Todo {
    pub fn is_done(&self) -> bool {
        self.state == State::Done
    }

    pub async fn load(
        ctx: &Context,
        pool: &sqlite::Pool,
        trip_id: Uuid,
    ) -> Result<Vec<Self>, Error> {
        let trip_id_param = trip_id.to_string();
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
    pub async fn find(
        ctx: &Context,
        pool: &sqlite::Pool,
        trip_id: Uuid,
        todo_id: Uuid,
    ) -> Result<Option<Self>, Error> {
        let trip_id_param = trip_id.to_string();
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

    #[tracing::instrument]
    pub async fn set_state(
        ctx: &Context,
        pool: &sqlite::Pool,
        trip_id: Uuid,
        todo_id: Uuid,
        state: State,
    ) -> Result<(), Error> {
        let user_id = ctx.user.id.to_string();
        let trip_id_param = trip_id.to_string();
        let todo_id_param = todo_id.to_string();
        let done = state == State::Done;

        let result = crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Update,
                component: sqlite::Component::Trips,
            },
            pool,
            r#"
                UPDATE trip_todos
                    SET done = ?
                WHERE trip_id = ?
                AND id = ?
                AND EXISTS(SELECT 1 FROM trips WHERE id = ? AND user_id = ?)"#,
            done,
            trip_id_param,
            todo_id_param,
            trip_id_param,
            user_id
        )
        .await?;

        (result.rows_affected() != 0).then_some(()).ok_or_else(|| {
            Error::Query(QueryError::NotFound {
                description: format!("todo {todo_id} not found for trip {trip_id}"),
            })
        })
    }
}
