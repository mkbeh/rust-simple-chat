use axum::{
    Json, extract::FromRequest, extract::rejection::JsonRejection, http::StatusCode,
    response::IntoResponse, response::Response,
};
use serde;
use serde::Serialize;
use std::{error::Error as StdError, fmt::Debug};
use thiserror::Error;
use validator::ValidationErrors;

use crate::core_utils::jwt;

pub trait ServiceError: Debug + StdError {
    fn status(&self) -> StatusCode;
    fn message(&self) -> String;
    fn field_as_string(&self) -> String;
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),

    #[error(transparent)]
    AuthError(#[from] jwt::JwtError),

    #[error(transparent)]
    DatabaseError(#[from] anyhow::Error),

    #[error("service error")]
    ServiceError(#[source] &'static dyn ServiceError),
}

impl ServerError {
    fn field_as_string(&self) -> String {
        match self {
            ServerError::JsonRejection(_) => String::from("JSON_REJECTION_ERROR"),
            ServerError::ValidationError(_) => String::from("VALIDATION_ERROR"),
            ServerError::DatabaseError(_) => String::from("DATABASE_ERROR"),
            ServerError::AuthError(_) => String::new(),
            ServerError::ServiceError(_) => String::new(),
        }
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ServerError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    for<'a> axum::Json<&'a T>: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(&self.0).into_response()
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            #[serde(rename = "message")]
            error_message: String,
            #[serde(rename = "type")]
            error_type: String,
        }

        let (status, error_message, error_type) = match &self {
            ServerError::JsonRejection(rejection) => (
                rejection.status(),
                rejection.body_text(),
                self.field_as_string(),
            ),

            ServerError::ValidationError(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("[{self}]").replace('\n', ", "),
                self.field_as_string(),
            ),

            ServerError::AuthError(jwt_err) => (
                jwt_err.to_status_code(),
                jwt_err.to_message(),
                jwt_err.field_as_string(),
            ),

            ServerError::ServiceError(err) => (err.status(), err.message(), err.field_as_string()),

            ServerError::DatabaseError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                err.to_string(),
                self.field_as_string(),
            ),
        };

        (
            status,
            Json(ErrorResponse {
                error_message,
                error_type,
            }),
        )
            .into_response()
    }
}
