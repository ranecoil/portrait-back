use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier, PasswordHash,
};
use chrono::{DateTime, Utc};
use sqlx::{query_as, FromRow, PgPool};
use uuid::Uuid;

use super::error::Error;

#[derive(FromRow)]
pub struct Creator {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub pfp: Option<String>,
    pw_hash: String,
    pub created: DateTime<Utc>
}

impl Creator {
    /// Create a new creator
    pub async fn new(
        name: String,
        email: String,
        pfp: Option<String>,
        password: String,
        db: &PgPool,
    ) -> Result<Self> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let pw_hash = argon2
            .hash_password(&password.into_bytes(), &salt)?
            .to_string();
        Ok(query_as!(
            Creator,
            "INSERT INTO creators (name, email, pfp, pw_hash) VALUES ($1, $2, $3, $4) RETURNING *",
            name,
            email,
            pfp,
            pw_hash
        )
        .fetch_one(db)
        .await?)
    }

    /// Get a single creator by their id
    pub async fn get_by_id(id: Uuid, db: &PgPool) -> Result<Self> {
        Ok(
            query_as!(Creator, "SELECT * FROM creators WHERE id = $1", id)
                .fetch_one(db)
                .await?,
        )
    }

    /// Lookup a creator by their name.
    pub async fn get_by_name(name: String, db: &PgPool) -> Result<Self> {
        let x = query_as!(Creator, "SELECT * FROM creators WHERE name = $1", name)
                    .fetch_one(db)
                    .await;
                match x {
                    Ok(x) => Ok(x),
                    Err(sqlx::Error::RowNotFound) => Err(Error::NotFound.into()),
                    Err(x) => Err(Error::InternalServerError(x.into()).into())
                }
    } 

    /// Lookup a creator by their email.
    pub async fn get_by_email(email: String, db: &PgPool) -> Result<Self>  {
            let x = query_as!(Creator, "SELECT * FROM creators WHERE email = $1", email)
                    .fetch_one(db)
                    .await;
                match x {
                    Ok(x) => Ok(x),
                    Err(sqlx::Error::RowNotFound) => Err(Error::NotFound.into()),
                    Err(x) => Err(Error::InternalServerError(x.into()).into())
                }
    } 

    /// Verify a creators password by their name
    pub async fn verify_by_name(name: String, password: String, db: &PgPool) -> Result<()> {
        Self::get_by_name(name, db).await?.verify(password)
    }

    /// Verify a creators password by their email
    pub async fn verify_by_email(email: String, password: String, db: &PgPool) -> Result<()> {
        Self::get_by_email(email, db).await?.verify(password)
    }

    fn verify(&self, password: String) -> Result<()> {
        Ok(Argon2::default().verify_password(&password.into_bytes(), &PasswordHash::new(&self.pw_hash)?)?)
    }

}