use actix_web::{body::BoxBody, http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("ALREADY_EXISTS")]
    AlreadyExists,
    #[error("INTERNAL_SERVER_ERROR")]
    InternalServerError(anyhow::Error),
    #[error("NOT_FOUND")]
    NotFound,
    #[error("UNAUTHORIZED")]
    Unauthorized,
}

#[derive(Debug, Error)]
#[error("{source}")]
pub struct ErrorResponse {
    source: anyhow::Error,
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> StatusCode {
        let source = &self.source;
        let status;

        if let Some(error) = source.downcast_ref::<ApiError>() {
            // Backend-native error
            status = match *error {
                ApiError::AlreadyExists => StatusCode::CONFLICT,
                ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::NotFound => StatusCode::NOT_FOUND,
                ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            }
        } else if let Some(error) = source.downcast_ref::<sqlx::Error>() {
            // SQLx error
            status = match *error {
                sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        } else {
            // Error has not been handled
            status = StatusCode::INTERNAL_SERVER_ERROR
        }

        status
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::new(self.status_code())
    }
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(err: anyhow::Error) -> Self {
        Self { source: err }
    }
}

impl From<ApiError> for ErrorResponse {
    fn from(err: ApiError) -> Self {
        Self { source: err.into() }
    }
}
