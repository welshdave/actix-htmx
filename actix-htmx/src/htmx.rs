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
pub struct HtmxDetails {
    inner: Rc<RefCell<HtmxDetailsInner>>,
    pub is_htmx: bool,
    pub boosted: bool,
    pub history_restore_request: bool,
}

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

enum DataType {
    String(Option<String>),
    Bool(bool),
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
    response_headers: IndexMap<String, String>,
    request_headers: IndexMap<String, DataType>,
}

impl HtmxDetailsInner {
    pub fn new(req: &HttpRequest) -> HtmxDetailsInner {
        let request_headers = collection![
            RequestHeaders::HX_REQUEST.to_string() => DataType::Bool(req.headers().get(RequestHeaders::HX_REQUEST).as_bool()),
            RequestHeaders::HX_BOOSTED.to_string() => DataType::Bool(req.headers().get(RequestHeaders::HX_BOOSTED).as_bool()),
            RequestHeaders::HX_CURRENT_URL.to_string() => DataType::String(req.headers().get(RequestHeaders::HX_CURRENT_URL).as_option_string()),
            RequestHeaders::HX_HISTORY_RESTORE_REQUEST.to_string() => DataType::Bool(req.headers().get(RequestHeaders::HX_HISTORY_RESTORE_REQUEST).as_bool()),
            RequestHeaders::HX_PROMPT.to_string() => DataType::String(req.headers().get(RequestHeaders::HX_PROMPT).as_option_string()),
            RequestHeaders::HX_TARGET.to_string() => DataType::String(req.headers().get(RequestHeaders::HX_TARGET).as_option_string()),
            RequestHeaders::HX_TRIGGER.to_string() => DataType::String(req.headers().get(RequestHeaders::HX_TRIGGER).as_option_string()),
            RequestHeaders::HX_TRIGGER_NAME.to_string() => DataType::String(req.headers().get(RequestHeaders::HX_TRIGGER_NAME).as_option_string()),
        ];

        HtmxDetailsInner {
            request_headers,
            response_headers: IndexMap::new(),
            standard_triggers: IndexMap::new(),
            after_settle_triggers: IndexMap::new(),
            after_swap_triggers: IndexMap::new(),
        }
    }

    fn get_bool_header(&self, header_name: &str) -> bool {
        self.request_headers
            .get(header_name)
            .map(|data_type| match data_type {
                DataType::Bool(b) => *b,
                _ => false,
            })
            .unwrap_or(false)
    }

    fn get_string_header(&self, header_name: &str) -> Option<String> {
        self.request_headers
            .get(header_name)
            .map(|data_type| match data_type {
                DataType::String(s) => {
                    if let Some(s) = s {
                        Some(s.clone())
                    } else {
                        None
                    }
                },
                _ => None,
            })
            .unwrap_or(None)
    }
}

impl HtmxDetails {
    fn from_inner(inner: Rc<RefCell<HtmxDetailsInner>>) -> HtmxDetails {
        let is_htmx = inner.borrow().get_bool_header(RequestHeaders::HX_REQUEST);
        let boosted = inner.borrow().get_bool_header(RequestHeaders::HX_BOOSTED);
        let history_restore_request = inner.borrow().get_bool_header(RequestHeaders::HX_HISTORY_RESTORE_REQUEST);

        HtmxDetails {
            inner,
            is_htmx,
            boosted,
            history_restore_request,
        }
    }

    pub fn new(req: &ServiceRequest) -> HtmxDetails {
        let req = req.request();
        let inner = Rc::new(RefCell::new(HtmxDetailsInner::new(req)));
        HtmxDetails::from_inner(inner)
    }

    pub fn current_url(&self) -> Option<String> {
        self.inner.borrow().get_string_header(RequestHeaders::HX_CURRENT_URL)
    }

    pub fn prompt(&self) -> Option<String> {
        self.inner.borrow().get_string_header(RequestHeaders::HX_PROMPT)
    }

    pub fn target(&self) -> Option<String> {
        self.inner.borrow().get_string_header(RequestHeaders::HX_TARGET)
    }

    pub fn trigger(&self) -> Option<String> {
        self.inner.borrow().get_string_header(RequestHeaders::HX_TRIGGER)
    }

    pub fn trigger_name(&self) -> Option<String> {
        self.inner.borrow().get_string_header(RequestHeaders::HX_TRIGGER_NAME)
    }

    pub fn trigger_event(&self, name: String, message: String, trigger_type: TriggerType) {
        match trigger_type {
            TriggerType::Standard => {
                self.inner.borrow_mut().standard_triggers.insert(name, message);
            }
            TriggerType::AfterSettle => {
                self.inner
                    .borrow_mut()
                    .after_settle_triggers
                    .insert(name, message);
            }
            TriggerType::AfterSwap => {
                self.inner
                    .borrow_mut()
                    .after_swap_triggers
                    .insert(name, message);
            }
        }
    }

    pub fn redirect(&self, path: String) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_REDIRECT.to_string(), path);
    }

    pub fn redirect_with_swap(&self, path: String) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_LOCATION.to_string(), path);
    }

    pub fn refresh(&self) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_REFRESH.to_string(), "true".to_string());
    }

    pub fn push_url(&self, path: String) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_PUSH_URL.to_string(), path);
    }

    pub fn replace_url(&self, path: String) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_REPLACE_URL.to_string(), path);
    }

    pub fn reswap(&self, swap_type: SwapType) {
        self.inner.borrow_mut().response_headers.insert(
            ResponseHeaders::HX_RESWAP.to_string(),
            swap_type.to_string(),
        );
    }

    pub fn retarget(&self, selector: String) {
        self.inner.borrow_mut().response_headers.insert(
            ResponseHeaders::HX_RETARGET.to_string(),
            selector.to_string(),
        );
    }

    pub fn reselect(&self, selector: String) {
        self.inner.borrow_mut().response_headers.insert(
            ResponseHeaders::HX_RESELECT.to_string(),
            selector.to_string(),
        );
    }

    pub(crate) fn get_triggers(&self, trigger_type: TriggerType) -> IndexMap<String, String> {
        match trigger_type {
            TriggerType::Standard => self.inner.borrow().standard_triggers.clone(),
            TriggerType::AfterSettle => self.inner.borrow().after_settle_triggers.clone(),
            TriggerType::AfterSwap => self.inner.borrow().after_swap_triggers.clone(),
        }
    }

    pub(crate) fn get_response_headers(&self) -> IndexMap<String, String> {
        self.inner.borrow().response_headers.clone()
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

        ready(Ok(HtmxDetails::from_inner(inner)))
    }
}

trait AsBool {
    fn as_bool(&self) -> bool;
}

trait AsOptionString {
    fn as_option_string(&self) -> Option<String>;
}

impl AsBool for Option<&HeaderValue> {
    fn as_bool(&self) -> bool {
        match self {
            Some(header) => {
                if let Ok(header) = header.to_str() {
                    header.parse::<bool>().unwrap_or(false)
                } else {
                    false
                }
            }
            None => false,
        }
    }
}

impl AsOptionString for Option<&HeaderValue> {
    fn as_option_string(&self) -> Option<String> {
        match self {
            Some(header) => {
                if let Ok(header) = header.to_str() {
                    Some(header.to_string())
                } else {
                    None
                }
            }
            None => None,
        }
    }
}