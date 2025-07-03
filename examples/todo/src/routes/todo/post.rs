use crate::domain::Todos;
use crate::routes::{HomeTemplate, TodosTemplate};
use crate::template_response::TemplateToResponse;
use actix_htmx::Htmx;
use actix_web::{web, HttpResponse, Responder};
use sqlx::{Pool, Sqlite};

#[derive(serde::Deserialize)]
pub struct NewTodo {
    name: String,
}

pub async fn create_todo(
    htmx: Htmx,
    form: web::Form<NewTodo>,
    pool: web::Data<Pool<Sqlite>>,
) -> impl Responder {
    let NewTodo { name } = form.0;

    match Todos::add_todo(&pool, &name).await {
        Ok(_) => {
            let todos = Todos::get_todos(&pool).await.unwrap_or_else(|_| {
                println!("Problem fetching todos!");
                Vec::default()
            });

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
