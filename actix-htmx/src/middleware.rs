use crate::{headers::ResponseHeaders, Htmx, TriggerType};

use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use indexmap::IndexMap;
use log::warn;
use std::future::{ready, Ready};

/// A middleware for Actix Web that handles htmx specific headers and triggers.
///
/// This module provides middleware functionality for using htmx in your Actix Web
/// application. It processes htmx headers and manages various types of triggers that
/// can be used for client-side interactions.
///
/// [`HtmxMiddleware`] injects an Htmx struct into any route that it wraps. This
/// Htmx struct provides helper properties and methods that allow for your application
/// to easily interact with htmx.
///
/// # Example
///
/// ```no_run
/// use actix_web::{web, App, HttpServer, Responder, HttpResponse};
/// use actix_htmx::{Htmx, HtmxMiddleware};
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     HttpServer::new(|| {
///         App::new()
///            .wrap(HtmxMiddleware)
///             .route("/", web::get().to(index))
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
///
/// async fn index(htmx: Htmx) -> impl Responder {
///     if !htmx.is_htmx {
///         HttpResponse::Ok().body(r##"
///             <!DOCTYPE html>
///             <html>
///                 <head>
///                     <title>htmx example</title>
///                     <script src="https://unpkg.com/htmx.org@2.0.5"></script>
///                 </head>
///                 <body>
///                     <div id="content">
///                         This was not an htmx request! <a href="/" hx-get="/" hx-target="#content">Make it htmx!</a>
///                     </div>
///                 </body>
///             </html>
///         "##)
///     } else {
///         HttpResponse::Ok().body(r##"
///         <div id="content">
///             This was an htmx request! <a href="/">Let's go back to plain old HTML</a>
///         <div>
///         "##)
///     }
/// }
/// ```
///
/// The middleware automatically processes the following htmx headers:
/// - `HX-Trigger`: For standard htmx triggers
/// - `HX-Trigger-After-Settle`: For triggers that fire after the settling phase
/// - `HX-Trigger-After-Swap`: For triggers that fire after content swap
///
pub struct HtmxMiddleware;

impl<S, B> Transform<S, ServiceRequest> for HtmxMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = InnerHtmxMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(InnerHtmxMiddleware { service }))
    }
}

#[doc(hidden)]
#[non_exhaustive]
pub struct InnerHtmxMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for InnerHtmxMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let htmx = Htmx::new(&req);

        req.extensions_mut().insert(htmx);

        let fut = self.service.call(req);

        Box::pin(async move {
            let res: ServiceResponse<B> = fut.await?;

            let (req, mut res) = res.into_parts();

            let trigger_json = |trigger_map: IndexMap<String, Option<String>>| -> String {
                let mut triggers = String::new();
                triggers.push('{');
                trigger_map.iter().for_each(|(key, value)| {
                    if let Some(value) = value {
                        if value.trim().starts_with('{') {
                            triggers.push_str(&format!("\"{}\": {},", key, value));
                        } else {
                            triggers.push_str(&format!("\"{}\": \"{}\",", key, value));
                        }
                    } else {
                        triggers.push_str(&format!("\"{}\": null,", key));
                    }
                });
                triggers.pop();
                triggers.push('}');
                triggers
            };

            let simple_header = |trigger_map: IndexMap<String, Option<String>>| -> String {
                let mut triggers = trigger_map
                    .iter()
                    .map(|(key, _)| key.to_string() + ",")
                    .collect::<String>();
                triggers.pop();
                triggers
            };

            let mut process_trigger_header =
                |header_name: HeaderName,
                 trigger_map: IndexMap<String, Option<String>>,
                 simple: bool| {
                    if trigger_map.is_empty() {
                        return;
                    }

                    let triggers = if simple {
                        simple_header(trigger_map)
                    } else {
                        trigger_json(trigger_map)
                    };

                    if let Ok(value) = HeaderValue::from_str(&triggers) {
                        res.headers_mut().insert(header_name, value);
                    } else {
                        warn!("Failed to parse {} header value: {}", header_name, triggers)
                    }
                };

            if let Some(htmx_response) = req.extensions().get::<Htmx>() {
                process_trigger_header(
                    HeaderName::from_static(ResponseHeaders::HX_TRIGGER),
                    htmx_response.get_triggers(TriggerType::Standard),
                    htmx_response.is_simple_trigger(TriggerType::Standard),
                );
                process_trigger_header(
                    HeaderName::from_static(ResponseHeaders::HX_TRIGGER_AFTER_SETTLE),
                    htmx_response.get_triggers(TriggerType::AfterSettle),
                    htmx_response.is_simple_trigger(TriggerType::AfterSettle),
                );
                process_trigger_header(
                    HeaderName::from_static(ResponseHeaders::HX_TRIGGER_AFTER_SWAP),
                    htmx_response.get_triggers(TriggerType::AfterSwap),
                    htmx_response.is_simple_trigger(TriggerType::AfterSwap),
                );

                let response_headers = htmx_response.get_response_headers();
                response_headers
                    .iter()
                    .for_each(|(key, value)| match key.parse() {
                        Ok(key) => {
                            if let Ok(value) = HeaderValue::from_str(value) {
                                res.headers_mut().insert(key, value);
                            } else {
                                warn!("Failed to parse {} header value: {}", key, value)
                            }
                        }
                        _ => {
                            warn!("Failed to parse header name: {}", key)
                        }
                    });
            }

            Ok(ServiceResponse::new(req, res))
        })
    }
}
