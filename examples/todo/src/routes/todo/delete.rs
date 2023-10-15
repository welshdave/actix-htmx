use crate::domain::Todos;
use crate::routes::TodosTemplate;
use actix_htmx::{HtmxDetails, TriggerType};
use actix_web::{http::header::ContentType, web, HttpResponse};
use askama::Template;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub async fn delete_todo(
    htmx_details: HtmxDetails,
    id: web::Path<Uuid>,
    pool: web::Data<Pool<Sqlite>>,
) -> HttpResponse {
    match Todos::delete_todo(&pool, *id).await {
        Ok(_) => {
            htmx_details.trigger_event("message".to_string(), "a task was deleted!".to_string(), TriggerType::Standard);
            let todos = Todos::get_todos(&pool).await.unwrap();
            let todo_template = TodosTemplate { todos: &todos };
            HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(todo_template.render().unwrap())
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
