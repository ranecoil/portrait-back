use actix_web::{
    http::Uri,
    web::{self, Data},
    App, HttpServer,
};
use aws_sdk_config::{Credentials, Endpoint, Region};
use aws_sdk_s3::Client;
use aws_types::credentials::SharedCredentialsProvider;
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
    s3_access_key: String,
    s3_secret_key: String,
    s3_endpoint: String,
    s3_bucket_name: String,
}

#[derive(Clone)]
pub struct State {
    pub db: PgPool,
    pub s3_client: Client,
    pub s3_bucket_name: String,
}

impl State {
    fn new(db: PgPool, s3_client: Client, s3_bucket_name: String) -> Self {
        Self {
            db,
            s3_client,
            s3_bucket_name,
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = envy::from_env::<Config>().expect("Couldn't load config from environment");

    // build aws s3 client
    let conf = aws_config::load_from_env().await;
    let ep = Endpoint::immutable(
        config
            .s3_endpoint
            .parse::<Uri>()
            .expect("Couldn't parse S3 URI"),
    );
    let cred_provider = SharedCredentialsProvider::new(Credentials::new(
        &config.s3_access_key,
        &config.s3_secret_key,
        None,
        None,
        "minio",
    ));
    let s3_conf = aws_sdk_s3::config::Builder::from(&conf)
        .endpoint_resolver(ep)
        .region(Region::new("pp-back-01"))
        .credentials_provider(cred_provider)
        .build();
    let s3_client = Client::from_conf(s3_conf);

    let db = PgPool::connect(&config.db_uri)
        .await
        .expect("Couldn't connect to database");
    migrate!("./migrations/")
        .run(&db)
        .await
        .expect("Couldn't run database migrations");

    let state = State::new(db, s3_client, config.s3_bucket_name);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .route("/version", web::get().to(routes::get_version))
            // Creator
            .route("/creator/signup", web::post().to(routes::creator::register))
            .route("/creator/login", web::post().to(routes::creator::login))
            .route("/creator/update", web::post().to(routes::creator::update))
            .route("/creator/pfp", web::post().to(routes::creator::upload_pfp))
        // currently locked for legal reasons (data preservation vs https://europa.eu/youreurope/citizens/consumers/internet-telecoms/data-protection-online-privacy/index_en.htm)
        //.route("/creator/delete", web::delete().to(routes::creator::delete))
    })
    .bind(config.host_uri)?
    .run()
    .await
}
