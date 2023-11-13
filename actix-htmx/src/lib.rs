//! htmx middleware for Actix Web.
//!
//! `actix-htmx` provides a method of easily working with htmx in actix web applications.
//! Wrap services with [`HtmxMiddleware`] to enable htmx support, and access the [`Htmx`]
//! extractor in your handlers to get information about the current htmx state. Helper methods also
//! exist to enable you to set htmx response headers, allowing easy triggering of htmx events from
//! server side code.
//!
//! # Getting Started
//! Register [`HtmxMiddleware`] on your `App`:
//!
//! ```no_run
//! use actix_htmx::{Htmx, HtmxMiddleware, TriggerType};
//! use actix_web::{web, App, HttpResponse, HttpServer, Responder};
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| {
//!         App::new()
//!             .wrap(HtmxMiddleware)
//!             .service(web::resource("/").to(index))
//!     })
//!     .bind("0.0.0.0:8080")?
//!     .run()
//!     .await
//! }
//!
//! async fn index(htmx: Htmx) -> impl Responder {
//!     if htmx.is_htmx {
//!         // build a partial view
//!     } else {
//!         // build a full view
//!     }
//!     htmx.trigger_event(
//!         "my_event".to_string(),
//!         Some(r#"{"level": "info", "message": "my event message!"}"#.to_string()),
//!         Some(TriggerType::Standard)
//!     );
//!
//!     HttpResponse::Ok().content_type("text/html").body(// render the view)
//!
//! }
//! ```

mod headers;
mod htmx;
mod middleware;

pub use self::{
    htmx::{Htmx, TriggerType},
    middleware::HtmxMiddleware,
};
