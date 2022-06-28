use actix_multipart::Multipart;
use actix_web::{
    web::{self},
    HttpResponse, Responder,
};
use crate::{
    models::{
        error::{ErrorResponse},
        session::Session, s3::upload,
    },
    State,
};

pub async fn upload_pfp(
    session: Session,
    payload: Multipart,
    state: web::Data<State>,
) -> Result<impl Responder, ErrorResponse> {
    upload(payload, &state.s3_client, &state.s3_bucket_name, format!("pfp-{}", session.subject), "pfp").await?;
    Ok(HttpResponse::Ok().finish())
}
