use actix_web::HttpResponse;
use askama::Template;

pub trait TemplateToResponse {
    fn to_response(&self) -> HttpResponse;
}

impl<T: Template> TemplateToResponse for T {
    fn to_response(&self) -> HttpResponse {
        match self.render() {
            Ok(body) => HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(body),
            Err(e) => {
                eprintln!("Template rendering error: {}", e);
                HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body(format!("Template rendering failed: {}", e))
            }
        }
    }
}
