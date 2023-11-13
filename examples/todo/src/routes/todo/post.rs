use crate::domain::Todos;

use crate::routes::{HomeTemplate, TodosTemplate};
use actix_htmx::Htmx;
use actix_web::{web, HttpResponse};
use askama_actix::TemplateToResponse;
use sqlx::{Pool, Sqlite};

#[derive(serde::Deserialize)]
pub struct NewTodo {
    name: String,
}

pub async fn create_todo(
    htmx: Htmx,
    form: web::Form<NewTodo>,
    pool: web::Data<Pool<Sqlite>>,
) -> HttpResponse {
    let NewTodo { name } = form.0;

    match Todos::add_todo(&pool, &name).await {
        Ok(_) => {
            let todos = match Todos::get_todos(&pool).await {
                Ok(x) => x,
                Err(_) => {
                    println!("Problem fetching todos!");
                    Vec::default()
                }
            };

            htmx.replace_url("/".to_string());

            if htmx.boosted {
                let todo_template = TodosTemplate { todos: &todos };
                todo_template.to_response()
            } else {
                let home = HomeTemplate { todos: &todos };
                home.to_response()
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
