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
//!     HttpResponse::Ok().content_type("text/html").body("<html><head>Test!</head><body>My Content</body></html>")
//!
//! }
//! ```

mod headers;
mod htmx;
mod middleware;

pub use self::{
    htmx::{Htmx, SwapType, TriggerType},
    middleware::HtmxMiddleware,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headers::ResponseHeaders;
    use actix_web::{
        http::header::HeaderName,
        test::{self, TestRequest},
        web, App, HttpResponse,
    };

    #[actix_web::test]
    async fn test_htmx_middleware_basic() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.trigger_event(
                    "test-event".to_string(),
                    Some("test-value".to_string()),
                    Some(TriggerType::Standard),
                );
                HttpResponse::Ok().finish()
            }),
        ))
        .await;

        let req = TestRequest::get()
            .uri("/test")
            .insert_header((HeaderName::from_static("hx-request"), "true"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let trigger_header = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_TRIGGER))
            .unwrap();
        assert!(trigger_header
            .to_str()
            .unwrap()
            .contains(r#""test-event": "test-value""#));
    }

    #[actix_web::test]
    async fn test_htmx_middleware_after_settle() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.trigger_event(
                    "settle-event".to_string(),
                    None,
                    Some(TriggerType::AfterSettle),
                );
                HttpResponse::Ok().finish()
            }),
        ))
        .await;

        let req = TestRequest::get()
            .uri("/test")
            .insert_header((HeaderName::from_static("hx-request"), "true"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let settle_header = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_TRIGGER_AFTER_SETTLE))
            .unwrap();
        
        assert!(settle_header
            .to_str()
            .unwrap()
            .contains("settle-event"));
    }

    #[actix_web::test]
    async fn test_htmx_request_detection() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                assert!(htmx.is_htmx);
                HttpResponse::Ok().finish()
            }),
        ))
        .await;

        let req = TestRequest::get()
            .uri("/test")
            .insert_header((HeaderName::from_static("hx-request"), "true"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_non_htmx_request() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                assert!(!htmx.is_htmx);
                HttpResponse::Ok().finish()
            }),
        ))
        .await;

        let req = TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_boosted_request() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                assert!(htmx.boosted);
                HttpResponse::Ok().finish()
            }),
        ))
        .await;

        let req = TestRequest::get()
            .uri("/test")
            .insert_header((HeaderName::from_static("hx-boosted"), "true"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
