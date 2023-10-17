use crate::domain::{Status, Todo};
use askama::Template;

mod delete;
mod post;
mod put;

pub use delete::*;
pub use post::*;
pub use put::*;

#[derive(Template)]
#[template(path = "../src/routes/todo/todos.html")]
pub struct TodosTemplate<'a> {
    pub todos: &'a Vec<Todo>,
}
