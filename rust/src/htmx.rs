use axum::http::header::{HeaderMap, HeaderName, HeaderValue};

pub enum Event {
    TripItemEdited,
}

impl From<Event> for HeaderValue {
    fn from(val: Event) -> Self {
        HeaderValue::from_static(val.to_str())
    }
}

impl Event {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::TripItemEdited => "TripItemEdited",
        }
    }
}

pub enum ResponseHeaders {
    Trigger,
    PushUrl,
}

impl From<ResponseHeaders> for HeaderName {
    fn from(val: ResponseHeaders) -> Self {
        match val {
            ResponseHeaders::Trigger => HeaderName::from_static("hx-trigger"),
            ResponseHeaders::PushUrl => HeaderName::from_static("hx-push-url"),
        }
    }
}

pub enum RequestHeaders {
    HtmxRequest,
}

impl From<RequestHeaders> for HeaderName {
    fn from(val: RequestHeaders) -> Self {
        match val {
            RequestHeaders::HtmxRequest => HeaderName::from_static("hx-request"),
        }
    }
}

#[tracing::instrument]
pub fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get::<HeaderName>(RequestHeaders::HtmxRequest.into())
        .map_or(false, |value| value == "true")
}
