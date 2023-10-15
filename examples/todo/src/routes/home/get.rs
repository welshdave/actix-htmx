use crate::domain::Todos;

use actix_web::{http::header::ContentType, web, HttpResponse};
use askama::Template;
use sqlx::{Pool, Sqlite};
use crate::routes::HomeTemplate;

pub async fn home(pool: web::Data<Pool<Sqlite>>) -> HttpResponse {
    match Todos::get_todos(&pool).await {
        Ok(todos) => {
            let home = HomeTemplate { todos: &todos };
            HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(home.render().unwrap())
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
