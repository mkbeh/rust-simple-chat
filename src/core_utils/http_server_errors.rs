use std::{fmt, fmt::Display};

use http::StatusCode;
use thiserror::Error;

use crate::core_utils::errors::ServiceError;

const UNHANDLED_ERROR: &str = "UNHANDLED_ERROR";
const METHOD_NOT_FOUND: &str = "METHOD_NOT_FOUND";
const METHOD_NOT_ALLOWED: &str = "METHOD_NOT_ALLOWED";

#[derive(Debug, Error)]
pub enum CommonServerErrors {
    Panic,
    MethodNotFound,
    MethodNotAllowed,
}

impl Display for CommonServerErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl ServiceError for CommonServerErrors {
    fn status(&self) -> StatusCode {
        match self {
            CommonServerErrors::Panic => StatusCode::INTERNAL_SERVER_ERROR,
            CommonServerErrors::MethodNotFound => StatusCode::NOT_FOUND,
            CommonServerErrors::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
        }
    }

    fn message(&self) -> String {
        match self {
            CommonServerErrors::Panic => "unhandled error".to_string(),
            CommonServerErrors::MethodNotFound => "method not found".to_string(),
            CommonServerErrors::MethodNotAllowed => "method not allowed".to_string(),
        }
    }

    fn field_as_string(&self) -> String {
        match self {
            CommonServerErrors::Panic => UNHANDLED_ERROR.to_string(),
            CommonServerErrors::MethodNotFound => METHOD_NOT_FOUND.to_string(),
            CommonServerErrors::MethodNotAllowed => METHOD_NOT_ALLOWED.to_string(),
        }
    }
}
