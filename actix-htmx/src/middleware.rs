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
                    }
                    else {
                        triggers.push_str(&format!("\"{}\": null,", key));
                    }
                });
                triggers.pop();
                triggers.push('}');
                triggers
            };

            let mut process_trigger_header =
                |header_name: HeaderName, trigger_map: IndexMap<String, Option<String>>| {
                    if trigger_map.is_empty() {
                        return;
                    }
                    let triggers = trigger_json(trigger_map);
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
                );
                process_trigger_header(
                    HeaderName::from_static(ResponseHeaders::HX_TRIGGER_AFTER_SETTLE),
                    htmx_response.get_triggers(TriggerType::AfterSettle),
                );
                process_trigger_header(
                    HeaderName::from_static(ResponseHeaders::HX_TRIGGER_AFTER_SWAP),
                    htmx_response.get_triggers(TriggerType::AfterSwap),
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
