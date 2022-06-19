use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("UNAUTHORIZED")]
    Unauthorized,
    #[error("INTERNAL_SERVER_ERROR")]
    InternalServerError(anyhow::Error),
    #[error("NOT_FOUND")]
    NotFound,
    #[error("ALREADY_EXISTS")]
    AlreadyExists,
}