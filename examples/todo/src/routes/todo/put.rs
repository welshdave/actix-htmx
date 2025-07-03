use crate::domain::{Status, Todos};
use crate::routes::TodosTemplate;
use crate::template_response::TemplateToResponse;
use actix_htmx::{Htmx, TriggerType};
use actix_web::{web, HttpResponse, Responder};
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
) -> impl Responder {
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
            let todos = Todos::get_todos(&pool).await.unwrap_or_else(|_| {
                println!("Problem fetching todos!");
                Vec::default()
            });
            let todo_template = TodosTemplate { todos: &todos };
            todo_template.to_response()
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
