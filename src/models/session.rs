use crate::{
    models::error::{ApiError, ErrorResponse},
    State,
};
use actix_web::{web::Data, FromRequest};
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, FromRow, PgPool};
use std::{future::Future, pin::Pin};
use uuid::Uuid;

#[derive(FromRow)]
pub struct Session {
    pub token: Uuid,
    pub subject: Uuid,
    pub created: DateTime<Utc>,
}

impl Session {
    /// Create a new session for a given user.
    pub async fn new(subject: Uuid, db: &PgPool) -> anyhow::Result<Self> {
        let session = query_as!(
            Session,
            "INSERT INTO sessions (subject) VALUES ($1) RETURNING *",
            subject
        )
        .fetch_one(db)
        .await?;

        Ok(session)
    }

    /// Get a session by its token.
    pub async fn get(token: Uuid, db: &PgPool) -> anyhow::Result<Self> {
        let session = query_as!(Session, "SELECT * FROM sessions WHERE token = $1", token)
            .fetch_one(db)
            .await?;

        Ok(session)
    }

    /// Get all currently active sessions by its subject.
    pub async fn get_by_subject(subject: Uuid, db: &PgPool) -> anyhow::Result<Vec<Self>> {
        let session = query_as!(
            Session,
            "SELECT * FROM sessions WHERE subject = $1",
            subject
        )
        .fetch_all(db)
        .await?;

        Ok(session)
    }

    /// Remove a single session.
    ///
    /// If you just want to remove a session and only got its `token` you're most likely searching for [`Session::remove_by_token()`]
    pub async fn remove(&self, db: &PgPool) -> anyhow::Result<()> {
        query!("DELETE FROM sessions WHERE token = $1", self.token)
            .execute(db)
            .await?;

        Ok(())
    }

    #[doc(alias = "remove")]
    /// Remove a single session by its token.
    pub async fn remove_by_token(token: Uuid, db: &PgPool) -> anyhow::Result<()> {
        query!("DELETE FROM sessions WHERE token = $1", token)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Remove all of the subjects currently known sessions.
    ///
    /// This will not return the number of affected rows and there's currently no way to do that w\ adding a dedicated function.
    pub async fn remove_by_subject(subject: Uuid, db: &PgPool) -> anyhow::Result<()> {
        query!("DELETE FROM sessions WHERE subject = $1", subject)
            .execute(db)
            .await?;

        Ok(())
    }
}

impl FromRequest for Session {
    type Error = ErrorResponse;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            let data = req.app_data::<Data<State>>().context("App Data missing")?;

            let header = req
                .headers()
                .get("Authorization")
                .ok_or(ApiError::Unauthorized)?
                .to_str()
                .context(ApiError::Unauthorized)?;

            let token = Uuid::parse_str(header).context(ApiError::Unauthorized)?;
            let session = Session::get(token, &data.db).await?;

            Ok(session)
        })
    }
}
