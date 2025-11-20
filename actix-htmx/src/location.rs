use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

use crate::SwapType;

/// Builder for `HX-Location` header bodies.
///
/// HX-Location lets you instruct htmx to perform a navigation without a full
/// page reload while still providing extra context (target selector, swap mode,
/// request headers, etc.). Use [`Htmx::redirect_with_location`](crate::Htmx::redirect_with_location)
/// to send the resulting header.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HxLocation {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    swap: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    handler: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    select: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    push: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replace: Option<String>,
}

impl HxLocation {
    /// Create a new HX-Location builder pointing to the provided path.
    pub fn new(path: impl Into<String>) -> Self {
        HxLocation {
            path: path.into(),
            target: None,
            source: None,
            event: None,
            swap: None,
            headers: BTreeMap::new(),
            values: None,
            handler: None,
            select: None,
            push: None,
            replace: None,
        }
    }

    /// Override which element receives the swap.
    pub fn target(mut self, selector: impl Into<String>) -> Self {
        self.target = Some(selector.into());
        self
    }

    /// Set the selector for the element that should be treated as the source.
    pub fn source(mut self, selector: impl Into<String>) -> Self {
        self.source = Some(selector.into());
        self
    }

    /// Specify an event name to trigger on the client before navigation.
    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Change the swap behaviour for the follow-up request.
    pub fn swap(mut self, swap: SwapType) -> Self {
        self.swap = Some(swap.to_string());
        self
    }

    /// Provide a custom client-side response handler.
    pub fn handler(mut self, handler: impl Into<String>) -> Self {
        self.handler = Some(handler.into());
        self
    }

    /// Restrict the response fragment that htmx should swap.
    pub fn select(mut self, selector: impl Into<String>) -> Self {
        self.select = Some(selector.into());
        self
    }

    /// Add a custom header to the follow-up request.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Extend the custom headers with any iterator of key/value pairs.
    pub fn headers<I, K, V>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.headers
            .extend(headers.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Provide custom values accessible to the follow-up request.
    pub fn values_json(mut self, values: Value) -> Self {
        self.values = Some(values);
        self
    }

    /// Prevent htmx from pushing a new history entry.
    pub fn disable_push(mut self) -> Self {
        self.push = Some(Value::Bool(false));
        self
    }

    /// Override the history push path for the follow-up request.
    pub fn push_path(mut self, path: impl Into<String>) -> Self {
        self.push = Some(Value::String(path.into()));
        self
    }

    /// Replace the browser history entry with the provided path.
    pub fn replace(mut self, path: impl Into<String>) -> Self {
        self.replace = Some(path.into());
        self
    }

    pub(crate) fn into_header_value(self) -> String {
        serde_json::to_string(&self).expect("HxLocation serialization failed")
    }
}
