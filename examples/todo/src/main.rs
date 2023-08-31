mod routes;
use crate::routes::*;

use actix_files::Files;
use actix_htmx::HtmxMiddleware;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use std::env;
use std::path::Path;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!(env!{"CARGO_MANIFEST_DIR"});
    HttpServer::new(|| {
        App::new()
            .service(Files::new("/static", Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/static"))))
            .wrap(HtmxMiddleware)
            .service(web::resource("/").to(home))
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}