//! # actix-htmx
//!
//! `actix-htmx` provides a comprehensive solution for building dynamic web applications with htmx and Actix Web.
//! It offers type-safe access to htmx request headers, easy response manipulation, and powerful event triggering capabilities.
//!
//! ## Features
//!
//! - **Request Detection**: Automatically detect htmx requests, boosted requests, and history restore requests
//! - **Header Access**: Type-safe access to all htmx request headers (current URL, target, trigger, prompt, etc.)
//! - **Event Triggering**: Trigger custom JavaScript events with optional data at different lifecycle stages
//! - **Response Control**: Full control over htmx behaviour with response headers (redirect, refresh, swap, retarget, etc.)
//! - **Type Safety**: Fully typed API leveraging Rust's type system for correctness
//! - **Zero Configuration**: Works out of the box with sensible defaults
//! - **Performance**: Minimal overhead with efficient header processing
//!
//! # Getting Started
//! Register [`HtmxMiddleware`] on your `App` and use the [`Htmx`] extractor in your handlers:
//!
//! ```no_run
//! use actix_htmx::{Htmx, HtmxMiddleware};
//! use actix_web::{web, App, HttpResponse, HttpServer, Responder};
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| {
//!         App::new()
//!             .wrap(HtmxMiddleware)
//!             .route("/", web::get().to(index))
//!     })
//!     .bind("127.0.0.1:8080")?
//!     .run()
//!     .await
//! }
//!
//! async fn index(htmx: Htmx) -> impl Responder {
//!     if htmx.is_htmx {
//!         // This is an htmx request - return partial HTML
//!         HttpResponse::Ok().body("<div>Partial content for htmx</div>")
//!     } else {
//!         // Regular request - return full page
//!         HttpResponse::Ok().body("<html><body><div>Full page content</div></body></html>")
//!     }
//! }
//! ```

mod headers;
mod htmx;
mod location;
mod middleware;
mod trigger_payload;

pub use self::{
    htmx::{Htmx, SwapType, TriggerType},
    location::HxLocation,
    middleware::HtmxMiddleware,
    trigger_payload::TriggerPayload,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{headers::ResponseHeaders, HxLocation};
    use actix_web::http::header::HeaderValue;
    use actix_web::{
        http::header::HeaderName,
        test::{self, TestRequest},
        web, App, HttpResponse,
    };
    use serde::Serialize;
    use serde_json::{json, Value};

    #[derive(Serialize)]
    struct LocationValues {
        id: u32,
    }

    #[actix_web::test]
    async fn test_htmx_middleware_basic() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.trigger_event(
                    "test-event",
                    Some(TriggerPayload::text("test-value")),
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
        let trigger_json: Value = serde_json::from_str(trigger_header.to_str().unwrap()).unwrap();
        assert_eq!(trigger_json["test-event"], "test-value");
    }

    #[actix_web::test]
    async fn test_htmx_middleware_after_settle() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.trigger_event("settle-event", None, Some(TriggerType::AfterSettle));
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
            .get(HeaderName::from_static(
                ResponseHeaders::HX_TRIGGER_AFTER_SETTLE,
            ))
            .unwrap();

        assert!(settle_header.to_str().unwrap().contains("settle-event"));
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

    #[actix_web::test]
    async fn test_htmx_reswap() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.reswap(SwapType::Delete);
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

        let reswap_header = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_RESWAP))
            .unwrap();

        assert_eq!(reswap_header.to_str().unwrap(), "delete");
    }

    #[actix_web::test]
    async fn test_multiple_triggers() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.trigger_event(
                    "event1",
                    Some(TriggerPayload::text("value1")),
                    Some(TriggerType::Standard),
                );
                htmx.trigger_event(
                    "event2",
                    Some(TriggerPayload::text("value2")),
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
            .unwrap()
            .to_str()
            .unwrap();

        assert!(trigger_header.contains("event1"));
        assert!(trigger_header.contains("value1"));
        assert!(trigger_header.contains("event2"));
        assert!(trigger_header.contains("value2"));
    }

    #[actix_web::test]
    async fn test_multiple_trigger_types() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.trigger_event(
                    "standard",
                    Some(TriggerPayload::text("value1")),
                    Some(TriggerType::Standard),
                );
                htmx.trigger_event(
                    "after_settle",
                    Some(TriggerPayload::text("value2")),
                    Some(TriggerType::AfterSettle),
                );
                htmx.trigger_event(
                    "after_swap",
                    Some(TriggerPayload::text("value3")),
                    Some(TriggerType::AfterSwap),
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

        // Check standard trigger
        let standard_header = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_TRIGGER))
            .unwrap()
            .to_str()
            .unwrap();
        assert!(standard_header.contains("standard"));
        assert!(standard_header.contains("value1"));

        // Check after settle trigger
        let after_settle_header = resp
            .headers()
            .get(HeaderName::from_static(
                ResponseHeaders::HX_TRIGGER_AFTER_SETTLE,
            ))
            .unwrap()
            .to_str()
            .unwrap();
        assert!(after_settle_header.contains("after_settle"));
        assert!(after_settle_header.contains("value2"));

        // Check after swap trigger
        let after_swap_header = resp
            .headers()
            .get(HeaderName::from_static(
                ResponseHeaders::HX_TRIGGER_AFTER_SWAP,
            ))
            .unwrap()
            .to_str()
            .unwrap();
        assert!(after_swap_header.contains("after_swap"));
        assert!(after_swap_header.contains("value3"));
    }

    #[actix_web::test]
    async fn test_multiple_simple_triggers() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/simple",
            web::get().to(|htmx: Htmx| async move {
                htmx.trigger_event("event1", None, None);
                htmx.trigger_event("event2", None, None);
                HttpResponse::Ok().finish()
            }),
        ))
        .await;

        let req = TestRequest::get()
            .uri("/simple")
            .insert_header((HeaderName::from_static("hx-request"), "true"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let trigger_header = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_TRIGGER))
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        assert_eq!(trigger_header, "event1,event2");
    }

    #[actix_web::test]
    async fn test_string_payload_that_looks_like_json() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.trigger_event(
                    "looks-like-json",
                    Some(TriggerPayload::text("{not: \"json\"")),
                    None,
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

        let trigger_json: Value = serde_json::from_str(trigger_header.to_str().unwrap()).unwrap();
        assert_eq!(
            trigger_json["looks-like-json"],
            Value::String("{not: \"json\"".to_string())
        );
    }

    #[actix_web::test]
    async fn test_json_payload_trigger() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                let payload = TriggerPayload::json(json!({"id": 1, "complete": false})).unwrap();
                htmx.trigger_event("json-event", Some(payload), None);
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

        let trigger_json: Value = serde_json::from_str(trigger_header.to_str().unwrap()).unwrap();
        assert_eq!(trigger_json["json-event"]["id"], 1);
        assert_eq!(trigger_json["json-event"]["complete"], false);
    }

    #[actix_web::test]
    async fn test_htmx_redirect() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.redirect("/new-location");
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

        let redirect_header = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_REDIRECT))
            .unwrap();

        assert_eq!(redirect_header.to_str().unwrap(), "/new-location");
    }

    #[actix_web::test]
    async fn test_htmx_redirect_with_swap() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.redirect_with_swap("/new-location");
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

        let location_header = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_LOCATION))
            .unwrap();

        assert_eq!(location_header.to_str().unwrap(), "/new-location");
    }

    #[actix_web::test]
    async fn test_htmx_redirect_with_location() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                let location = HxLocation::new("/builder")
                    .target("#content")
                    .source("#button")
                    .event("custom")
                    .swap(SwapType::OuterHtml)
                    .handler("handleResponse")
                    .select(".fragment")
                    .header("X-Test", "1")
                    .values(LocationValues { id: 42 })
                    .expect("static payload should serialize")
                    .push_path("/history-path")
                    .replace("/replace-path");
                htmx.redirect_with_location(location);
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

        let location_header = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_LOCATION))
            .unwrap();
        let parsed: Value = serde_json::from_str(location_header.to_str().unwrap()).unwrap();
        assert_eq!(parsed["path"], "/builder");
        assert_eq!(parsed["target"], "#content");
        assert_eq!(parsed["source"], "#button");
        assert_eq!(parsed["event"], "custom");
        assert_eq!(parsed["swap"], "outerHTML");
        assert_eq!(parsed["handler"], "handleResponse");
        assert_eq!(parsed["select"], ".fragment");
        assert_eq!(parsed["headers"]["X-Test"], "1");
        assert_eq!(parsed["values"]["id"], 42);
        assert_eq!(parsed["push"], "/history-path");
        assert_eq!(parsed["replace"], "/replace-path");
    }

    #[actix_web::test]
    async fn test_url_methods() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.push_url("/pushed-url");
                htmx.replace_url("/replaced-url");
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

        let push_url = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_PUSH_URL))
            .unwrap();
        assert_eq!(push_url.to_str().unwrap(), "/pushed-url");

        let replace_url = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_REPLACE_URL))
            .unwrap();
        assert_eq!(replace_url.to_str().unwrap(), "/replaced-url");
    }

    #[actix_web::test]
    async fn test_target_methods() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.retarget("#new-target");
                htmx.reselect("#new-selection");
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

        let retarget = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_RETARGET))
            .unwrap();
        assert_eq!(retarget.to_str().unwrap(), "#new-target");

        let reselect = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_RESELECT))
            .unwrap();
        assert_eq!(reselect.to_str().unwrap(), "#new-selection");
    }

    #[actix_web::test]
    async fn test_request_information() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                assert_eq!(htmx.current_url().unwrap(), "http://example.com");
                assert_eq!(htmx.prompt().unwrap(), "test prompt");
                assert_eq!(htmx.target().unwrap(), "#target");
                assert_eq!(htmx.trigger().unwrap(), "click");
                assert_eq!(htmx.trigger_name().unwrap(), "button1");
                HttpResponse::Ok().finish()
            }),
        ))
        .await;

        let req = TestRequest::get()
            .uri("/test")
            .insert_header((HeaderName::from_static("hx-request"), "true"))
            .insert_header((
                HeaderName::from_static("hx-current-url"),
                "http://example.com",
            ))
            .insert_header((HeaderName::from_static("hx-prompt"), "test prompt"))
            .insert_header((HeaderName::from_static("hx-target"), "#target"))
            .insert_header((HeaderName::from_static("hx-trigger"), "click"))
            .insert_header((HeaderName::from_static("hx-trigger-name"), "button1"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_refresh() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                htmx.refresh();
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

        let refresh = resp
            .headers()
            .get(HeaderName::from_static(ResponseHeaders::HX_REFRESH))
            .unwrap();
        assert_eq!(refresh.to_str().unwrap(), "true");
    }

    #[actix_web::test]
    async fn test_malformed_headers() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx: Htmx| async move {
                // Should not panic and return None for malformed headers
                assert_eq!(htmx.current_url(), None);
                assert_eq!(htmx.prompt(), None);
                assert_eq!(htmx.target(), None);
                // Should not panic and should return false
                assert_eq!(htmx.is_htmx, false);
                HttpResponse::Ok().finish()
            }),
        ))
        .await;

        let req = TestRequest::get()
            .uri("/test")
            .insert_header((
                HeaderName::from_static("hx-current-url"),
                HeaderValue::from_bytes(b"\xFF\xFF").unwrap(),
            ))
            .insert_header((
                HeaderName::from_static("hx-prompt"),
                HeaderValue::from_bytes(b"\xFF\xFF").unwrap(),
            ))
            .insert_header((
                HeaderName::from_static("hx-target"),
                HeaderValue::from_bytes(b"\xFF\xFF").unwrap(),
            ))
            .insert_header((
                HeaderName::from_static("hx-request"),
                HeaderValue::from_bytes(b"\xFF\xFF").unwrap(),
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_from_request_with_extensions() {
        let app = test::init_service(App::new().wrap(HtmxMiddleware).route(
            "/test",
            web::get().to(|htmx1: Htmx, htmx2: Htmx| async move {
                // Both instances should be the same when retrieved from extensions
                assert_eq!(htmx1.is_htmx, htmx2.is_htmx);
                assert_eq!(htmx1.boosted, htmx2.boosted);
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
}
