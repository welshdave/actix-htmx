use crate::domain::{Status, Todo};
use askama::Template;

mod get;

pub use get::*;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate<'a> {
    pub todos: &'a [Todo],
}
