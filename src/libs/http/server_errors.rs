use std::{collections::HashMap, fmt, fmt::Display, sync::LazyLock};

use http::StatusCode;
use thiserror::Error;

use crate::libs::http::errors::ServiceError;

struct FullError {
    error_type: String,
    error_message: String,
    code: StatusCode,
}

static ERRORS: LazyLock<HashMap<InternalServerErrors, FullError>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    map.insert(
        InternalServerErrors::Panic,
        FullError {
            error_type: "UNHANDLED_ERROR".to_string(),
            error_message: "unhandled error".to_string(),
            code: StatusCode::INTERNAL_SERVER_ERROR,
        },
    );

    map.insert(
        InternalServerErrors::MethodNotFound,
        FullError {
            error_type: "METHOD_NOT_FOUND".to_string(),
            error_message: "method not found".to_string(),
            code: StatusCode::NOT_FOUND,
        },
    );

    map.insert(
        InternalServerErrors::MethodNotAllowed,
        FullError {
            error_type: "METHOD_NOT_ALLOWED".to_string(),
            error_message: "method not allowed".to_string(),
            code: StatusCode::METHOD_NOT_ALLOWED,
        },
    );

    map
});

#[derive(Debug, Error, PartialEq, Eq, Hash)]
pub enum InternalServerErrors {
    Panic,
    MethodNotFound,
    MethodNotAllowed,
}

impl Display for InternalServerErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl ServiceError for InternalServerErrors {
    fn status(&self) -> StatusCode {
        ERRORS
            .get(self)
            .map_or(StatusCode::INTERNAL_SERVER_ERROR, |e| e.code)
    }

    fn message(&self) -> String {
        ERRORS
            .get(self)
            .map_or("unknown error".to_string(), |e| e.error_message.to_string())
    }

    fn field_as_string(&self) -> String {
        ERRORS
            .get(self)
            .map_or("UNKNOWN_ERROR".to_string(), |e| e.error_type.to_string())
    }
}
