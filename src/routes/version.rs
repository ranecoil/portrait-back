use actix_web::HttpResponse;
use serde_json::json;

use crate::API_VERSION;

pub async fn get_version() -> HttpResponse {
    HttpResponse::Ok().json(json!({ "version": API_VERSION }))
}
