use core::fmt;
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

#[derive(Debug)]
pub struct InternalApplicationError {
    message: String,
}

impl Display for InternalApplicationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for InternalApplicationError {}

impl InternalApplicationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for InternalApplicationError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            self.message().to_string(),
        )
            .into_response()
    }
}
