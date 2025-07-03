use crate::domain::Todos;
use crate::routes::HomeTemplate;
use crate::template_response::TemplateToResponse;
use actix_web::{web, HttpResponse, Responder};
use sqlx::{Pool, Sqlite};

pub async fn home(pool: web::Data<Pool<Sqlite>>) -> impl Responder {
    match Todos::get_todos(&pool).await {
        Ok(todos) => {
            let home = HomeTemplate { todos: &todos };
            home.to_response()
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
