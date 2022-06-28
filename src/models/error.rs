use actix_web::{body::BoxBody, http::StatusCode, HttpResponse, ResponseError};
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("ALREADY_EXISTS")]
    AlreadyExists,
    #[error("BAD_REQUEST")]
    BadRequest,
    #[error("INTERNAL_SERVER_ERROR")]
    InternalServerError,
    #[error("NOT_FOUND")]
    NotFound,
    #[error("UNAUTHORIZED")]
    Unauthorized,
}

#[derive(Debug)]
pub struct ErrorResponse {
    inner: anyhow::Error,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<E> From<E> for ErrorResponse
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        let inner = error.into();
        ErrorResponse { inner }
    }
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> StatusCode {
        let source = &self.inner;
        let status;

        if let Some(error) = source.downcast_ref::<ApiError>() {
            // Backend-native error
            status = match *error {
                ApiError::AlreadyExists => StatusCode::CONFLICT,
                ApiError::BadRequest => StatusCode::BAD_REQUEST,
                ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
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
