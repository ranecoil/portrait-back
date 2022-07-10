use actix_multipart::Multipart;
use actix_web::rt;
use aws_sdk_s3::{Client, Config, Credentials, Endpoint};
use futures_util::{StreamExt, TryStreamExt};
use hyper::Body;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::models::error::ApiError;

pub async fn split_json<D>(mut multipart: Multipart) -> anyhow::Result<(D, Option<Body>)>
where
    D: DeserializeOwned,
{
    let mut data = None;
    let mut file = None;

    while let Some(f) = multipart.next().await {
        let mut field = f?;

        match field.name() {
            "data" => {
                let json_slice = field
                    .try_fold(Vec::new(), |mut acc, b| async move {
                        acc.extend_from_slice(b.as_ref());
                        Ok(acc)
                    })
                    .await?;

                let json: D = serde_json::from_slice(&json_slice)?;
                data = Some(json);
            }
            "file" => {
                let (mut sender, body) = Body::channel();

                rt::spawn(async move {
                    while let Some(b) = field.next().await {
                        if let Ok(bytes) = b {
                            sender.send_data(bytes).await.ok();
                        } else {
                            sender.abort();
                            break;
                        }
                    }
                });

                file = Some(body);
            }
            _ => (),
        }
    }

    Ok((data.ok_or(ApiError::MultipartMissingData)?, file))
}

#[derive(Clone)]
pub struct S3 {
    bucket: String,
    client: Client,
}

impl S3 {
    pub fn new(
        access_key: &str,
        secret_key: &str,
        endpoint: &str,
        bucket: &str,
    ) -> anyhow::Result<Self> {
        let config = Config::builder()
            .credentials_provider(Credentials::new(
                access_key, secret_key, None, None, "minio",
            ))
            .endpoint_resolver(Endpoint::immutable(endpoint.parse()?))
            .build();

        let bucket = bucket.to_string();
        let client = Client::from_conf(config);
        let s3 = S3 { bucket, client };

        Ok(s3)
    }

    pub async fn put<B>(&self, file: Body, key: &Uuid) -> anyhow::Result<()> {
        self.client
            .put_object()
            .body(file.into())
            .key(key.to_string())
            .send()
            .await?;

        Ok(())
    }
}
