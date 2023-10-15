use example_todo::routes::*;

use actix_files::Files;
use actix_htmx::HtmxMiddleware;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Pool, Sqlite};

use std::env;
use std::path::Path;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connection_pool = match init_db("todo.db").await {
        Ok(pool) => pool,
        Err(e) => {
            println!("Failed to connect to database: {}", e);
            return Ok(());
        }
    };

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(connection_pool.clone()))
            .service(Files::new(
                "/static",
                Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
            ))
            .wrap(HtmxMiddleware)
            .service(web::scope("/")
                .route("", web::get().to(home)))
            .service(
                web::scope("/todo")
                    .service(web::resource("").route(web::post().to(create_todo)))
                    .service(
                        web::resource("{id}")
                            .route(web::put().to(complete_todo))
                            .route(web::delete().to(delete_todo)),
                    ),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn init_db(file_name: &str) -> anyhow::Result<Pool<Sqlite>> {
    let db_full_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/data"))
        .join(file_name)
        .to_string_lossy()
        .to_string();

    let connection_string = format!("{}{}", "sqlite://", db_full_path);
    if !Path::new(&db_full_path).exists() {
        Sqlite::create_database(connection_string.as_str()).await?;
    }

    let db_pool = SqlitePool::connect(connection_string.as_str()).await?;

    sqlx::migrate!().run(&db_pool).await?;

    Ok(db_pool)
}
