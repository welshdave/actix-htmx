use actix_web::dev::{Payload, ServiceRequest};
use actix_web::error::Error;
use actix_web::http::header::HeaderValue;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{ready, Ready};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::headers::{RequestHeaders, ResponseHeaders};

macro_rules! collection {
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
}

#[derive(Clone)]
pub struct HtmxDetails(Rc<RefCell<HtmxDetailsInner>>);

#[derive(Clone, PartialEq, Eq)]
pub enum TriggerType {
    Standard,
    AfterSettle,
    AfterSwap,
}

pub enum SwapType {
    InnerHtml,
    OuterHtml,
    BeforeBegin,
    AfterBegin,
    BeforeEnd,
    AfterEnd,
    Delete,
    None,
}

impl fmt::Display for SwapType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwapType::InnerHtml => write!(f, "innerHTML"),
            SwapType::OuterHtml => write!(f, "outerHTML"),
            SwapType::BeforeBegin => write!(f, "beforebegin"),
            SwapType::AfterBegin => write!(f, "afterbegin"),
            SwapType::BeforeEnd => write!(f, "beforeend"),
            SwapType::AfterEnd => write!(f, "afterend"),
            SwapType::Delete => write!(f, "delete"),
            SwapType::None => write!(f, "none"),
        }
    }
}

struct HtmxDetailsInner {
    standard_triggers: IndexMap<String, String>,
    after_settle_triggers: IndexMap<String, String>,
    after_swap_triggers: IndexMap<String, String>,
    request_headers: IndexMap<String, Option<String>>,
    response_headers: IndexMap<String, String>,
}

impl HtmxDetailsInner {
    pub fn new(req: &HttpRequest) -> HtmxDetailsInner {
        let extract_string = |header: Option<&HeaderValue>| -> Option<String> {
            match header {
                Some(header) => {
                    if let Ok(header) = header.to_str() {
                        Some(header.to_string())
                    } else {
                        None
                    }
                }
                None => None,
            }
        };

        let request_headers = collection![
            RequestHeaders::HX_REQUEST.to_string() => extract_string(req.headers().get(RequestHeaders::HX_REQUEST)),
            RequestHeaders::HX_BOOSTED.to_string() => extract_string(req.headers().get(RequestHeaders::HX_BOOSTED)),
            RequestHeaders::HX_CURRENT_URL.to_string() => extract_string(req.headers().get(RequestHeaders::HX_CURRENT_URL)),
            RequestHeaders::HX_HISTORY_RESTORE_REQUEST.to_string() => extract_string(req.headers().get(RequestHeaders::HX_HISTORY_RESTORE_REQUEST)),
            RequestHeaders::HX_PROMPT.to_string() => extract_string(req.headers().get(RequestHeaders::HX_PROMPT)),
            RequestHeaders::HX_TARGET.to_string() => extract_string(req.headers().get(RequestHeaders::HX_TARGET)),
            RequestHeaders::HX_TRIGGER.to_string() => extract_string(req.headers().get(RequestHeaders::HX_TRIGGER)),
            RequestHeaders::HX_TRIGGER_NAME.to_string() => extract_string(req.headers().get(RequestHeaders::HX_TRIGGER_NAME)),
        ];

        HtmxDetailsInner {
            request_headers,
            response_headers: IndexMap::new(),
            standard_triggers: IndexMap::new(),
            after_settle_triggers: IndexMap::new(),
            after_swap_triggers: IndexMap::new(),
        }
    }
}

impl HtmxDetails {
    pub fn new(req: &ServiceRequest) -> HtmxDetails {
        let req = req.request();
        let inner = Rc::new(RefCell::new(HtmxDetailsInner::new(req)));
        HtmxDetails(inner)
    }

    pub fn is_htmx(&self) -> bool {
        HtmxDetails::extract_bool(
            self.0.borrow().request_headers[&RequestHeaders::HX_REQUEST.to_string()].clone(),
        )
    }

    pub fn boosted(&self) -> bool {
        HtmxDetails::extract_bool(
            self.0.borrow().request_headers[&RequestHeaders::HX_BOOSTED.to_string()].clone(),
        )
    }

    pub fn current_url(&self) -> Option<String> {
        self.0.borrow().request_headers[&RequestHeaders::HX_CURRENT_URL.to_string()].clone()
    }

    pub fn history_restore_request(&self) -> bool {
        HtmxDetails::extract_bool(
            self.0.borrow().request_headers
                [&RequestHeaders::HX_HISTORY_RESTORE_REQUEST.to_string()]
                .clone(),
        )
    }

    pub fn prompt(&self) -> Option<String> {
        self.0.borrow().request_headers[&RequestHeaders::HX_PROMPT.to_string()].clone()
    }

    pub fn target(&self) -> Option<String> {
        self.0.borrow().request_headers[&RequestHeaders::HX_TARGET.to_string()].clone()
    }

    pub fn trigger(&self) -> Option<String> {
        self.0.borrow().request_headers[&RequestHeaders::HX_TRIGGER.to_string()].clone()
    }

    pub fn trigger_name(&self) -> Option<String> {
        self.0.borrow().request_headers[&RequestHeaders::HX_TRIGGER_NAME.to_string()].clone()
    }

    pub fn trigger_event(&self, name: String, message: String, trigger_type: TriggerType) {
        match trigger_type {
            TriggerType::Standard => {
                self.0.borrow_mut().standard_triggers.insert(name, message);
            }
            TriggerType::AfterSettle => {
                self.0
                    .borrow_mut()
                    .after_settle_triggers
                    .insert(name, message);
            }
            TriggerType::AfterSwap => {
                self.0
                    .borrow_mut()
                    .after_swap_triggers
                    .insert(name, message);
            }
        }
    }

    pub fn redirect(&self, path: String) {
        self.0
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_REDIRECT.to_string(), path);
    }

    pub fn redirect_with_swap(&self, path: String) {
        self.0
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_LOCATION.to_string(), path);
    }

    pub fn refresh(&self) {
        self.0
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_REFRESH.to_string(), "true".to_string());
    }

    pub fn push_url(&self, path: String) {
        self.0
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_PUSH_URL.to_string(), path);
    }

    pub fn replace_url(&self, path: String) {
        self.0
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_REPLACE_URL.to_string(), path);
    }

    pub fn reswap(&self, swap_type: SwapType) {
        self.0.borrow_mut().response_headers.insert(
            ResponseHeaders::HX_RESWAP.to_string(),
            swap_type.to_string(),
        );
    }

    pub fn retarget(&self, selector: String) {
        self.0.borrow_mut().response_headers.insert(
            ResponseHeaders::HX_RETARGET.to_string(),
            selector.to_string(),
        );
    }

    pub fn reselect(&self, selector: String) {
        self.0.borrow_mut().response_headers.insert(
            ResponseHeaders::HX_RESELECT.to_string(),
            selector.to_string(),
        );
    }

    pub(crate) fn get_triggers(&self, trigger_type: TriggerType) -> IndexMap<String, String> {
        match trigger_type {
            TriggerType::Standard => self.0.borrow().standard_triggers.clone(),
            TriggerType::AfterSettle => self.0.borrow().after_settle_triggers.clone(),
            TriggerType::AfterSwap => self.0.borrow().after_swap_triggers.clone(),
        }
    }

    pub(crate) fn get_response_headers(&self) -> IndexMap<String, String> {
        self.0.borrow().response_headers.clone()
    }

    fn extract_bool(value: Option<String>) -> bool {
        if let Some(value) = value {
            value == "true"
        } else {
            false
        }
    }
}

impl FromRequest for HtmxDetails {
    type Error = Error;
    type Future = Ready<Result<HtmxDetails, Error>>;

    #[inline]
    fn from_request(req: &actix_web::HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(htmx_details) = req.extensions_mut().get::<HtmxDetails>() {
            return ready(Ok(htmx_details.clone()));
        }

        let inner = Rc::new(RefCell::new(HtmxDetailsInner::new(req)));

        ready(Ok(HtmxDetails(inner)))
    }
}
