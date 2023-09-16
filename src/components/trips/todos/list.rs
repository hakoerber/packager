use maud::{html, Markup};
use uuid::Uuid;

use super::Todo;
use crate::components::view;
use crate::models::trips::Trip;

#[derive(Debug)]
pub struct List<'a> {
    pub trip: &'a Trip,
    pub todos: &'a Vec<Todo>,
}

#[derive(Debug)]
pub struct BuildInput {
    pub edit_todo: Option<Uuid>,
}

impl<'a> view::View for List<'a> {
    type Input = BuildInput;

    #[tracing::instrument]
    fn build(&self, input: Self::Input) -> Markup {
        html!(
            div #todolist {
                h1 ."text-xl" ."mb-5" { "Todos" }
                ul
                    ."flex"
                    ."flex-col"
                {
                    @for todo in self.todos {
                        @let state = input.edit_todo
                            .map(|id| if todo.id == id {
                                super::UiState::Edit
                            } else {
                                super::UiState::Default
                            }).unwrap_or(super::UiState::Default);
                        (todo.build(super::BuildInput{trip_id:self.trip.id, state}))
                    }
                    (NewTodo::build(&self.trip.id))
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
