use actix_multipart::Multipart;
use aws_sdk_s3::Client;
use tokio_stream::StreamExt;

use super::error::{ApiError, ErrorResponse};

pub async fn extract_multipart_data(mut payload: Multipart) -> Result<Vec<u8>, ApiError> {
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

    Ok(data)
}

pub async fn upload(
    data: Vec<u8>,
    client: &Client,
    bucket_name: &String,
    mut key: String,
    kind: &str,
    allowed_content: Option<Vec<&str>>
) -> Result<(), ErrorResponse> {
    let mime_type = infer::get(&data).ok_or(ApiError::BadRequest)?;
    let mime_string = mime_type.to_string();
    
    if let Some(types) = allowed_content {
        if !types.contains(&mime_string.as_str()) {
            return Err(ApiError::BadRequest.into());
        }
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
        .map_err(|_| ApiError::InternalServerError)?;
    Ok(())
}
