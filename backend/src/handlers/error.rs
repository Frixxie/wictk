use core::fmt;
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub struct ApplicationError {
    message: String,
    status_code: StatusCode,
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ApplicationError {}

impl ApplicationError {
    pub fn new(message: &str, status_code: StatusCode) -> Self {
        Self {
            message: message.into(),
            status_code,
        }
    }
}

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        (self.status_code, self.message.to_string()).into_response()
    }
}
