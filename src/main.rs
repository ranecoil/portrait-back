use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use serde::Deserialize;
use sqlx::{migrate, PgPool};

mod models;
mod routes;

const API_VERSION: &str = "v1";

#[derive(Deserialize)]
struct Config {
    host_uri: String,
    db_uri: String,
}

#[derive(Clone)]
pub struct State {
    pub db: PgPool,
}

impl State {
    fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("{:#?}", error),
    };

    let db = PgPool::connect(&config.db_uri).await.unwrap();
    migrate!("./migrations/")
        .run(&db)
        .await
        .expect("Couldn't run database migrations.");

    let state = State::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/version", web::get().to(routes::get_version))
    })
    .bind(config.host_uri)?
    .run()
    .await
}
