use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use dotenv::dotenv;
use serde::Deserialize;
use sqlx::{migrate, PgPool};

use models::s3::S3;

mod models;
mod routes;

const API_VERSION: &str = "v1";

#[derive(Deserialize)]
struct Config {
    host_uri: String,
    database_url: String,
    s3_access_key: String,
    s3_secret_key: String,
    s3_endpoint: String,
    s3_bucket: String,
}

#[derive(Clone)]
pub struct State {
    pub db: PgPool,
    pub s3: S3,
}

impl State {
    fn new(db: PgPool, s3: S3) -> Self {
        Self { db, s3 }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = envy::from_env::<Config>().expect("Couldn't load config from environment");

    let db = PgPool::connect(&config.database_url)
        .await
        .expect("Couldn't connect to database");
    migrate!("./migrations/")
        .run(&db)
        .await
        .expect("Couldn't run database migrations");

    let s3 = S3::new(
        &config.s3_access_key,
        &config.s3_secret_key,
        &config.s3_endpoint,
        &config.s3_bucket,
    )
    .expect("Couldn't build S3 client");

    let state = State::new(db, s3);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .route("/version", web::get().to(routes::get_version))
            // Creator
            .route("/creator/signup", web::post().to(routes::creator::register))
            .route("/creator/login", web::post().to(routes::creator::login))
            .route("/creator/update", web::post().to(routes::creator::update))
        // currently locked for legal reasons (data preservation vs https://europa.eu/youreurope/citizens/consumers/internet-telecoms/data-protection-online-privacy/index_en.htm)
        //.route("/creator/delete", web::delete().to(routes::creator::delete))
    })
    .bind(config.host_uri)?
    .run()
    .await
}
