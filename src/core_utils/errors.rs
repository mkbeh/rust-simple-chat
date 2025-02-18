use axum::extract::rejection::JsonRejection;
use axum::extract::FromRequest;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use serde;
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),
    #[error(transparent)]
    DatabaseError(#[from] anyhow::Error),
}

impl ServerError {
    fn field_as_string(&self) -> String {
        match self {
            ServerError::JsonRejection(_) => String::from("JSON_REJECTION_ERROR"),
            ServerError::ValidationError(_) => String::from("VALIDATION_ERROR"),
            ServerError::DatabaseError(_) => String::from("DATABASE_ERROR"),
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

        let error_type = self.field_as_string();

        let (status, error_message) = match self {
            ServerError::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            ServerError::ValidationError(_) => {
                let err_msg = format!("[{self}]").replace('\n', ", ");
                (StatusCode::UNPROCESSABLE_ENTITY, err_msg)
            }
            ServerError::DatabaseError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
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
