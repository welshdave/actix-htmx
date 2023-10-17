use crate::domain::Todos;
use crate::routes::TodosTemplate;
use actix_htmx::{HtmxDetails, TriggerType};
use actix_web::{web, HttpResponse};
use askama_actix::{TemplateToResponse};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub async fn delete_todo(
    htmx_details: HtmxDetails,
    id: web::Path<Uuid>,
    pool: web::Data<Pool<Sqlite>>,
) -> HttpResponse {
    match Todos::delete_todo(&pool, *id).await {
        Ok(_) => {
            htmx_details.trigger_event(
                "message".to_string(),
                format!("Task with id {} was deleted", id).to_string(),
                TriggerType::Standard,
            );
            let todos = Todos::get_todos(&pool).await.unwrap();
            let todo_template = TodosTemplate { todos: &todos };
            todo_template.to_response()
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
