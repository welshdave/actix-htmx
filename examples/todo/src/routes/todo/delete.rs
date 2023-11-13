use crate::domain::Todos;
use crate::routes::TodosTemplate;
use actix_htmx::{Htmx, TriggerType};
use actix_web::{web, HttpResponse};
use askama_actix::TemplateToResponse;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub async fn delete_todo(
    htmx: Htmx,
    id: web::Path<Uuid>,
    pool: web::Data<Pool<Sqlite>>,
) -> HttpResponse {
    match Todos::delete_todo(&pool, *id).await {
        Ok(_) => {
            htmx.trigger_event(
                "message".to_string(),
                Some(format!("Task with id {} was deleted", id).to_string()),
                Some(TriggerType::Standard),
            );
            htmx.trigger_event(
                "message2".to_string(),
                Some("Just showing you can trigger more than one event".to_string()),
                None,
            );
            htmx.trigger_event(
                "message".to_string(),
                Some("Another event, just for fun".to_string()),
                Some(TriggerType::AfterSettle),
            );
            htmx.trigger_event("deleted".to_string(), None, None);
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
