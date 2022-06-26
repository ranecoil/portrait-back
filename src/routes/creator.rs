use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{creator::Creator, error::ErrorResponse, session::Session},
    State,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub user_identifier: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}
pub async fn login(
    req: web::Json<LoginRequest>,
    state: web::Data<State>,
) -> Result<HttpResponse, ErrorResponse> {
    let creator = Creator::get_by_name(req.user_identifier.clone(), &state.db).await?;
    let res = HttpResponse::Ok().json(LoginResponse {
        token: Session::new(creator.id, &state.db).await?.token.to_string(),
    });
    Ok(res)
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub token: Uuid,
}

pub async fn register(
    req: web::Json<RegisterRequest>,
    state: web::Data<State>,
) -> Result<HttpResponse, ErrorResponse> {
    let x = Creator::new(&req.username, &req.email, None, &req.password, &state.db).await?;
    let token = Session::new(x.id, &state.db).await?.token;
    let res = HttpResponse::Ok().json(RegisterResponse { token });
    Ok(res)
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: String,
    pub new_password: Option<String>,
    pub pfp: Option<String>,
}

pub async fn update(
    session: Session,
    req: web::Json<UpdateUserRequest>,
    state: web::Data<State>,
) -> Result<HttpResponse, ErrorResponse> {
    let x = Creator::update(
        &session.subject,
        req.email.as_ref(),
        &req.password,
        req.new_password.as_ref(),
        req.pfp.as_ref(),
        &state.db,
    )
    .await;
    dbg!(x);
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
