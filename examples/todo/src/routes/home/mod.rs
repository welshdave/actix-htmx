use actix_web::{http::header::ContentType, HttpRequest, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "../src/routes/home/home.html")]
struct Home;

pub async fn home(req: HttpRequest) -> HttpResponse {
    let s = Home.render().unwrap();
    HttpResponse::Ok().content_type(ContentType::html()).body(s)
}
