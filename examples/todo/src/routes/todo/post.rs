use crate::domain::Todos;

use crate::routes::{HomeTemplate, TodosTemplate};
use actix_htmx::HtmxDetails;
use actix_web::{web, HttpResponse};
use askama_actix::{TemplateToResponse};
use sqlx::{Pool, Sqlite};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
}

pub async fn create_todo(
    htmx_details: HtmxDetails,
    form: web::Form<FormData>,
    pool: web::Data<Pool<Sqlite>>,
) -> HttpResponse {
    let FormData { name } = form.0;

    match Todos::add_todo(&pool, &name).await {
        Ok(_) => {
            let todos = Todos::get_todos(&pool).await.unwrap();

            htmx_details.replace_url("/".to_string());

            if htmx_details.boosted() {
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
