use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{creator::Creator, error::ErrorResponse, s3::split_json, session::Session},
    State,
};

#[derive(Deserialize)]
pub struct SignInRequest {
    pub name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SignInResponse {
    pub token: String,
}
pub async fn sign_in(
    req: web::Json<SignInRequest>,
    state: web::Data<State>,
) -> Result<HttpResponse, ErrorResponse> {
    let creator = Creator::get_by_name(req.name.clone(), &state.db).await?;
    let res = HttpResponse::Ok().json(SignInResponse {
        token: Session::new(creator.id, &state.db).await?.token.to_string(),
    });
    Ok(res)
}

#[derive(Deserialize)]
pub struct SignUpRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SignUpResponse {
    pub token: Uuid,
}

pub async fn sign_up(
    req: web::Json<SignUpRequest>,
    state: web::Data<State>,
) -> Result<HttpResponse, ErrorResponse> {
    let x = Creator::new(&req.name, &req.email, None, &req.password, &state.db).await?;
    let token = Session::new(x.id, &state.db).await?.token;
    let res = HttpResponse::Ok().json(SignUpResponse { token });
    Ok(res)
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
}

pub async fn update(
    session: Session,
    req: Multipart,
    state: web::Data<State>,
) -> Result<HttpResponse, ErrorResponse> {
    let (data, picture): (UpdateUserRequest, _) = split_json(req).await?;

    Creator::update(
        &session.subject,
        data.email.as_deref(),
        data.password.as_deref(),
        picture.as_ref(),
        &state.db,
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub password: String,
}

pub async fn delete(
    req: web::Json<DeleteRequest>,
    session: Session,
    state: web::Data<State>,
) -> Result<HttpResponse, ErrorResponse> {
    Creator::verify_by_id(&session.subject, &req.password, &state.db).await?;
    Session::remove_by_subject(&session.subject, &state.db).await?;
    Creator::delete_by_id(&session.subject, &state.db).await?;
    let res = HttpResponse::NoContent().finish();
    Ok(res)
}
