use actix_web::dev::{Payload, ServiceRequest};
use actix_web::error::Error;
use actix_web::http::header::HeaderValue;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{ready, Ready};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::{
    headers::{RequestHeaders, ResponseHeaders},
    trigger_payload::TriggerPayload,
    HxLocation,
};

/// Provides access to htmx request information and methods for setting htmx response headers.
///
/// The [`Htmx`] struct serves two main purposes:
/// 1. As an extractor, providing information about the current htmx request
/// 2. For managing htmx response headers
///
/// # Request Information
///
/// Access information about the current request:
/// ```rust
/// use actix_web::{get, HttpResponse, Responder};
/// use actix_htmx::Htmx;
///
/// #[get("/")]
/// async fn handler(htmx: Htmx) -> impl Responder {
///     if htmx.is_htmx {
///         // This is an htmx request
///         println!("Target element: {}", htmx.target().unwrap_or_default());
///         println!("Trigger element: {}", htmx.trigger().unwrap_or_default());
///     }
///     // ...
///     HttpResponse::Ok()
/// }
/// ```
///
/// # Response Headers
///
/// Set htmx response headers for client-side behaviour:
/// ```rust
/// use actix_web::{post, HttpResponse, Responder};
/// use actix_htmx::{Htmx, SwapType, TriggerPayload, TriggerType};
/// use serde_json::json;
///
/// #[post("/create")]
/// async fn create(htmx: Htmx) -> impl Responder {
///     // Trigger a client-side event
///     let payload = TriggerPayload::json(json!({ "id": 123 })).unwrap();
///     htmx.trigger_event(
///         "itemCreated",
///         Some(payload),
///         Some(TriggerType::Standard)
///     );
///
///     // Change how content is swapped
///     htmx.reswap(SwapType::OuterHtml);
///
///     // Redirect after the request
///     htmx.redirect("/items");
///
///     // ...
///     HttpResponse::Ok()
/// }
/// ```
///
#[derive(Clone)]
pub struct Htmx {
    inner: Rc<RefCell<HtmxInner>>,
    /// True if the request was made by htmx (has the `hx-request` header)
    pub is_htmx: bool,
    /// True if the request was made by a boosted element (has the `hx-boosted` header)
    pub boosted: bool,
    /// True if this is a history restore request (has the `hx-history-restore-request` header)
    pub history_restore_request: bool,
}

macro_rules! collection {
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
}

/// Specifies when an htmx event should be triggered.
///
/// Events can be triggered at different points in the htmx request lifecycle.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TriggerType {
    Standard,
    AfterSettle,
    AfterSwap,
}

/// Specifies how htmx should swap content into the target element.
///
/// These correspond to the different swap strategies available in htmx.
pub enum SwapType {
    /// Replace the inner HTML of the target element (default)
    InnerHtml,
    /// Replace the entire target element
    OuterHtml,
    /// Insert content before the target element
    BeforeBegin,
    /// Insert content at the beginning of the target element
    AfterBegin,
    /// Insert content at the end of the target element
    BeforeEnd,
    /// Insert content after the target element
    AfterEnd,
    /// Delete the target element
    Delete,
    /// Don't swap any content
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

struct HtmxInner {
    standard_triggers: IndexMap<String, Option<TriggerPayload>>,
    after_settle_triggers: IndexMap<String, Option<TriggerPayload>>,
    after_swap_triggers: IndexMap<String, Option<TriggerPayload>>,
    response_headers: IndexMap<String, String>,
    request_headers: IndexMap<String, DataType>,
    simple_trigger: HashMap<TriggerType, bool>,
}

impl HtmxInner {
    pub fn new(req: &HttpRequest) -> HtmxInner {
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

        HtmxInner {
            request_headers,
            response_headers: IndexMap::new(),
            standard_triggers: IndexMap::new(),
            after_settle_triggers: IndexMap::new(),
            after_swap_triggers: IndexMap::new(),
            simple_trigger: HashMap::new(),
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
                DataType::String(s) => s.clone(),
                _ => None,
            })
            .unwrap_or(None)
    }
}

impl Htmx {
    fn from_inner(inner: Rc<RefCell<HtmxInner>>) -> Htmx {
        let is_htmx = inner.borrow().get_bool_header(RequestHeaders::HX_REQUEST);
        let boosted = inner.borrow().get_bool_header(RequestHeaders::HX_BOOSTED);
        let history_restore_request = inner
            .borrow()
            .get_bool_header(RequestHeaders::HX_HISTORY_RESTORE_REQUEST);

        Htmx {
            inner,
            is_htmx,
            boosted,
            history_restore_request,
        }
    }

    pub fn new(req: &ServiceRequest) -> Htmx {
        let req = req.request();
        let inner = Rc::new(RefCell::new(HtmxInner::new(req)));
        Htmx::from_inner(inner)
    }

    /// Get the current URL from the `hx-current-url` header.
    ///
    /// This header is sent by htmx and contains the current URL of the page.
    pub fn current_url(&self) -> Option<String> {
        self.inner
            .borrow()
            .get_string_header(RequestHeaders::HX_CURRENT_URL)
    }

    /// Get the user's response to an `hx-prompt` from the `hx-prompt` header.
    ///
    /// This header contains the user's input when an htmx request includes a prompt.
    pub fn prompt(&self) -> Option<String> {
        self.inner
            .borrow()
            .get_string_header(RequestHeaders::HX_PROMPT)
    }

    /// Get the ID of the target element from the `hx-target` header.
    ///
    /// This header contains the ID of the element that will be updated with the response.
    pub fn target(&self) -> Option<String> {
        self.inner
            .borrow()
            .get_string_header(RequestHeaders::HX_TARGET)
    }

    /// Get the ID of the element that triggered the request from the `hx-trigger` header.
    ///
    /// This header contains the ID of the element that initiated the htmx request.
    pub fn trigger(&self) -> Option<String> {
        self.inner
            .borrow()
            .get_string_header(RequestHeaders::HX_TRIGGER)
    }

    /// Get the name of the element that triggered the request from the `hx-trigger-name` header.
    ///
    /// This header contains the name attribute of the element that initiated the htmx request.
    pub fn trigger_name(&self) -> Option<String> {
        self.inner
            .borrow()
            .get_string_header(RequestHeaders::HX_TRIGGER_NAME)
    }

    /// Trigger a custom JavaScript event on the client side.
    ///
    /// This method allows you to trigger custom events that can be listened to with JavaScript.
    /// Events can include optional data and can be triggered at different points in the htmx lifecycle.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the event to trigger
    /// * `payload` - Optional data to send with the event (typically JSON)
    /// * `trigger_type` - When to trigger the event (defaults to `Standard`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use actix_htmx::{Htmx, TriggerPayload, TriggerType};
    ///
    /// fn handler(htmx: Htmx) {
    ///     // Simple event without data
    ///     htmx.trigger_event("item-deleted", None, None);
    ///
    ///     // Event with JSON data
    ///     let payload = TriggerPayload::json(serde_json::json!({
    ///         "message": "Success!",
    ///         "type": "info"
    ///     })).unwrap();
    ///
    ///     htmx.trigger_event(
    ///         "notification",
    ///         Some(payload),
    ///         Some(TriggerType::Standard)
    ///     );
    /// }
    /// ```
    pub fn trigger_event(
        &self,
        name: impl Into<String>,
        payload: Option<TriggerPayload>,
        trigger_type: Option<TriggerType>,
    ) {
        let name = name.into();
        let trigger_type = trigger_type.unwrap_or(TriggerType::Standard);
        let mut inner = self.inner.borrow_mut();

        if payload.is_some() {
            inner.simple_trigger.insert(trigger_type.clone(), false);
        }

        let target_map = match trigger_type {
            TriggerType::Standard => &mut inner.standard_triggers,
            TriggerType::AfterSettle => &mut inner.after_settle_triggers,
            TriggerType::AfterSwap => &mut inner.after_swap_triggers,
        };

        target_map.insert(name, payload);
    }

    /// Redirect to a new page with a full page reload.
    ///
    /// This sets the `hx-redirect` header, which causes htmx to perform a client-side redirect
    /// to the specified URL with a full page reload.
    pub fn redirect(&self, path: impl Into<String>) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_REDIRECT.to_string(), path.into());
    }

    /// Redirect to a new page using htmx (no full page reload).
    ///
    /// This sets the `hx-location` header, which causes htmx to make a new request
    /// to the specified URL and swap the response into the current page.
    pub fn redirect_with_swap(&self, path: impl Into<String>) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_LOCATION.to_string(), path.into());
    }

    /// Redirect using a fully customized HX-Location object.
    ///
    /// This lets you control additional behaviour like target selectors,
    /// swap strategies, or custom values for the follow-up request.
    /// Build the payload with [`HxLocation`](crate::HxLocation).
    pub fn redirect_with_location(&self, location: HxLocation) {
        self.inner.borrow_mut().response_headers.insert(
            ResponseHeaders::HX_LOCATION.to_string(),
            location.into_header_value(),
        );
    }

    /// Refresh the current page.
    ///
    /// This sets the `hx-refresh` header, which causes htmx to refresh the entire page.
    pub fn refresh(&self) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_REFRESH.to_string(), "true".to_string());
    }

    /// Update the browser URL without causing a navigation.
    ///
    /// This sets the `hx-push-url` header, which updates the browser's address bar
    /// and adds an entry to the browser history.
    pub fn push_url(&self, path: impl Into<String>) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_PUSH_URL.to_string(), path.into());
    }

    /// Replace the current URL in the browser history.
    ///
    /// This sets the `hx-replace-url` header, which updates the browser's address bar
    /// without adding a new entry to the browser history.
    pub fn replace_url(&self, path: impl Into<String>) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_REPLACE_URL.to_string(), path.into());
    }

    /// Change how htmx swaps content into the target element.
    ///
    /// This sets the `hx-reswap` header, which overrides the default swap behaviour
    /// for this response.
    pub fn reswap(&self, swap_type: SwapType) {
        self.inner.borrow_mut().response_headers.insert(
            ResponseHeaders::HX_RESWAP.to_string(),
            swap_type.to_string(),
        );
    }

    /// Change the target element for content swapping.
    ///
    /// This sets the `hx-retarget` header, which changes which element
    /// the response content will be swapped into.
    pub fn retarget(&self, selector: impl Into<String>) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_RETARGET.to_string(), selector.into());
    }

    /// Select specific content from the response to swap.
    ///
    /// This sets the `hx-reselect` header, which allows you to select
    /// a subset of the response content to swap into the target.
    pub fn reselect(&self, selector: impl Into<String>) {
        self.inner
            .borrow_mut()
            .response_headers
            .insert(ResponseHeaders::HX_RESELECT.to_string(), selector.into());
    }

    pub(crate) fn get_triggers(
        &self,
        trigger_type: TriggerType,
    ) -> IndexMap<String, Option<TriggerPayload>> {
        match trigger_type {
            TriggerType::Standard => self.inner.borrow().standard_triggers.clone(),
            TriggerType::AfterSettle => self.inner.borrow().after_settle_triggers.clone(),
            TriggerType::AfterSwap => self.inner.borrow().after_swap_triggers.clone(),
        }
    }

    pub(crate) fn is_simple_trigger(&self, trigger_type: TriggerType) -> bool {
        match trigger_type {
            TriggerType::Standard => *self
                .inner
                .borrow()
                .simple_trigger
                .get(&TriggerType::Standard)
                .unwrap_or(&true),
            TriggerType::AfterSettle => *self
                .inner
                .borrow()
                .simple_trigger
                .get(&TriggerType::AfterSettle)
                .unwrap_or(&true),
            TriggerType::AfterSwap => *self
                .inner
                .borrow()
                .simple_trigger
                .get(&TriggerType::AfterSwap)
                .unwrap_or(&true),
        }
    }

    pub(crate) fn get_response_headers(&self) -> IndexMap<String, String> {
        self.inner.borrow().response_headers.clone()
    }
}

impl FromRequest for Htmx {
    type Error = Error;
    type Future = Ready<Result<Htmx, Error>>;

    #[inline]
    fn from_request(req: &actix_web::HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(htmx) = req.extensions_mut().get::<Htmx>() {
            return ready(Ok(htmx.clone()));
        }

        let inner = Rc::new(RefCell::new(HtmxInner::new(req)));

        ready(Ok(Htmx::from_inner(inner)))
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
