use axum::{
    body::Body,
    http::Response,
    response::{IntoResponse, Json},
};
use http::StatusCode;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ErrorMessage {
    pub status: &'static str,
    pub message: String,
}

pub struct ErrorResponse(StatusCode, Json<ErrorMessage>);

impl Default for ErrorResponse {
    fn default() -> Self {
        ErrorResponse(
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMessage {
                status: "fail",
                message: "Internal server error".to_string(),
            }),
        )
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response<Body> {
        (self.0, self.1).into_response()
    }
}

impl ErrorResponse {
    pub fn custom_error(status: StatusCode, message: &str) -> Self {
        ErrorResponse(
            status,
            Json(ErrorMessage {
                status: "fail",
                message: String::from(message),
            }),
        )
    }
    pub fn custom_error_message(message: &str) -> Self {
        ErrorResponse(
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMessage {
                status: "fail",
                message: String::from(message),
            }),
        )
    }
}
