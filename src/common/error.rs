use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::error::Error as MongoError;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub message: String,
    pub details: Option<Value>,
}

impl AppError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details<T>(status: StatusCode, message: impl Into<String>, details: T) -> Self
    where
        T: Serialize,
    {
        Self {
            status,
            message: message.into(),
            details: Some(json!(details)),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn validation<T>(message: impl Into<String>, details: T) -> Self
    where
        T: Serialize,
    {
        Self::with_details(StatusCode::BAD_REQUEST, message, details)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn database_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "success": false,
            "error": {
                "code": self.status.as_u16(),
                "message": self.message,
                "details": self.details,
            }
        }));

        (self.status, body).into_response()
    }
}

impl From<MongoError> for AppError {
    fn from(_: MongoError) -> Self {
        AppError::database_error()
    }
}
