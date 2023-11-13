use crate::domain::{Status, Todos};
use crate::routes::TodosTemplate;
use actix_htmx::{Htmx, TriggerType};
use actix_web::{web, HttpResponse};
use askama_actix::TemplateToResponse;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct ToDoStatus {
    completed: Option<String>,
}

pub async fn update_todo(
    htmx: Htmx,
    id: web::Path<Uuid>,
    form: web::Form<ToDoStatus>,
    pool: web::Data<Pool<Sqlite>>,
) -> HttpResponse {
    let ToDoStatus { completed } = form.0;

    let status = if let None = completed {
        Status::Pending
    } else {
        Status::Done
    };

    match Todos::update_todo(&pool, status, *id).await {
        Ok(_) => {
            htmx.trigger_event(
                "message".to_string(),
                Some(format!("Task with id {} was set to {}", id, status).to_string()),
                Some(TriggerType::Standard),
            );
            let todos = match Todos::get_todos(&pool).await {
                Ok(x) => x,
                Err(_) => {
                    println!("Problem fetching todos!");
                    Vec::default()
                }
            };
            let todo_template = TodosTemplate { todos: &todos };
            todo_template.to_response()
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
