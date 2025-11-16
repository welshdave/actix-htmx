use serde::Serialize;
use serde_json::{to_value, Value};

/// A strongly typed payload for HX-Trigger* headers.
///
/// Construct instances through helper constructors, then pass them to
/// [`Htmx::trigger_event`](crate::Htmx::trigger_event)
/// to make sure payloads are always valid JSON.
#[derive(Clone, Debug)]
pub struct TriggerPayload {
    inner: Value,
}

impl TriggerPayload {
    /// Create a payload from any serializable value.
    pub fn json<T>(value: T) -> serde_json::Result<Self>
    where
        T: Serialize,
    {
        to_value(value).map(Self::from_value)
    }

    /// Create a payload directly from a `serde_json::Value`.
    pub fn from_value(value: Value) -> Self {
        TriggerPayload { inner: value }
    }

    /// Convenience helper for string payloads.
    pub fn text(value: impl Into<String>) -> Self {
        TriggerPayload::from_value(Value::String(value.into()))
    }

    /// Convenience helper for boolean payloads.
    pub fn boolean(value: bool) -> Self {
        TriggerPayload::from_value(Value::Bool(value))
    }

    /// Convenience helper for numeric payloads.
    pub fn number<N>(value: N) -> serde_json::Result<Self>
    where
        N: Serialize,
    {
        TriggerPayload::json(value)
    }

    pub(crate) fn as_json_value(&self) -> Value {
        self.inner.clone()
    }
}
