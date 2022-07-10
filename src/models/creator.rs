use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use chrono::{DateTime, Utc};
use hyper::body::Body;
use sqlx::{query, query_as, FromRow, PgPool};
use uuid::Uuid;

use crate::models::error::ApiError;

#[derive(FromRow, Debug)]
pub struct Creator {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub pfp: Option<String>,
    pw_hash: String,
    pub created: DateTime<Utc>,
}

impl Creator {
    /// Create a new creator
    pub async fn new(
        name: &String,
        email: &String,
        pfp: Option<String>,
        password: &String,
        db: &PgPool,
    ) -> anyhow::Result<Self> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let pw_hash = argon2
            .hash_password(&password.to_owned().into_bytes(), &salt)?
            .to_string();

        let creator = query_as!(
            Creator,
            "INSERT INTO creators (name, email, pfp, pw_hash) VALUES ($1, $2, $3, $4) RETURNING *",
            name,
            email,
            pfp,
            pw_hash
        )
        .fetch_one(db)
        .await?;

        Ok(creator)
    }

    /// Get a single creator by their id
    pub async fn get_by_id(id: Uuid, db: &PgPool) -> anyhow::Result<Self> {
        let creator = query_as!(Creator, "SELECT * FROM creators WHERE id = $1", id)
            .fetch_one(db)
            .await?;

        Ok(creator)
    }

    /// Lookup a creator by their name.
    pub async fn get_by_name(name: String, db: &PgPool) -> anyhow::Result<Self> {
        let creator = query_as!(Creator, "SELECT * FROM creators WHERE name = $1", name)
            .fetch_one(db)
            .await?;

        Ok(creator)
    }

    /// Lookup a creator by their email.
    pub async fn get_by_email(email: String, db: &PgPool) -> anyhow::Result<Self> {
        let creator = query_as!(Creator, "SELECT * FROM creators WHERE email = $1", email)
            .fetch_one(db)
            .await?;

        Ok(creator)
    }

    pub async fn verify_by_id(id: &Uuid, password: &String, db: &PgPool) -> anyhow::Result<()> {
        Self::get_by_id(*id, db).await?.verify(password)
    }

    /// Verify a creators password by their name
    pub async fn verify_by_name(
        name: String,
        password: &String,
        db: &PgPool,
    ) -> anyhow::Result<()> {
        Self::get_by_name(name, db).await?.verify(password)
    }

    /// Verify a creators password by their email
    pub async fn verify_by_email(
        email: String,
        password: &String,
        db: &PgPool,
    ) -> anyhow::Result<()> {
        Self::get_by_email(email, db).await?.verify(password)
    }

    fn verify(&self, password: &String) -> anyhow::Result<()> {
        Argon2::default()
            .verify_password(password.as_bytes(), &PasswordHash::new(&self.pw_hash)?)
            .map_err(|e| match e {
                argon2::password_hash::Error::Password => ApiError::Unauthorized,
                _ => ApiError::InternalServerError,
            })?;
        Ok(())
    }

    pub async fn update(
        id: &Uuid,
        email: Option<&str>,
        password: Option<&str>,
        picture: Option<&Body>,
        db: &PgPool,
    ) -> anyhow::Result<Creator> {
        password
            .map::<anyhow::Result<_>, _>(|p| {
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                let pw_hash = argon2.hash_password(p.as_bytes(), &salt)?.to_string();
                Ok(pw_hash)
            })
            .transpose()?;

        let creator = query_as!(Creator,
            "UPDATE creators SET email = COALESCE($1, name), pw_hash = COALESCE($2, pw_hash) WHERE id = $3 RETURNING *",
            email, password, id)
            .fetch_one(db)
            .await?;

        Ok(creator)
    }

    pub async fn delete_by_id(id: &Uuid, db: &PgPool) -> anyhow::Result<()> {
        query!("DELETE FROM creators WHERE id = $1", id)
            .execute(db)
            .await?;
        Ok(())
    }
}
