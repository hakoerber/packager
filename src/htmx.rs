use axum::http::header::{HeaderMap, HeaderName, HeaderValue};

pub enum Event {
    TripItemEdited,
}

impl From<Event> for HeaderValue {
    fn from(val: Event) -> Self {
        Self::from_static(val.to_str())
    }
}

impl Event {
    #[must_use]
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
            ResponseHeaders::Trigger => Self::from_static("hx-trigger"),
            ResponseHeaders::PushUrl => Self::from_static("hx-push-url"),
        }
    }
}

pub enum RequestHeaders {
    HtmxRequest,
}

impl From<RequestHeaders> for HeaderName {
    fn from(val: RequestHeaders) -> Self {
        match val {
            RequestHeaders::HtmxRequest => Self::from_static("hx-request"),
        }
    }
}

#[tracing::instrument]
pub fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get::<HeaderName>(RequestHeaders::HtmxRequest.into())
        .is_some_and(|value| value == "true")
}
