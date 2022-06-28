use actix_multipart::Multipart;
use aws_sdk_s3::Client;
use tokio_stream::StreamExt;

use super::error::{ErrorResponse, ApiError};

pub async fn upload(mut payload: Multipart, client: &Client, bucket_name: &String, mut key: String, kind: &str) -> Result<(), ErrorResponse> {
    let mut field = match payload.try_next().await {
        Ok(field) => match field {
            Some(field) => field,
            None => {
                return Err(ApiError::InternalServerError.into());
            }
        },
        Err(e) => {
            dbg!(e);
            return Err(ApiError::BadRequest.into());
        }
    };
    let mut data = Vec::new();
    while let Some(chunk) = field.next().await {
        let x: actix_web::web::Bytes = chunk.unwrap();
        data.append(&mut x.to_vec());
    }

    let mime_type = infer::get(&data).ok_or(ApiError::BadRequest)?;
    let mime_string = mime_type.to_string();

    // only permit webp and png files => TODO: jpg to png conversion?
    if !(mime_string.eq("image/webp") || mime_string.eq("image/png")) {
        return Err(ApiError::BadRequest.into());
    } else if (data.len() / 1024 / 1024) > 3 {
        // check file size > ~3mb
        return Err(ApiError::BadRequest.into());
    }

    // append correct file extension if not present
    if !key.ends_with(mime_type.extension()) {
        key = format!("{}.{}", key, mime_type.extension());
    }

    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(data.into())
        .content_type(mime_string)
        .metadata("pp-type", kind)
        .send()
        .await
        .map_err(|_| {
            ApiError::InternalServerError
        })?;
    Ok(())
}