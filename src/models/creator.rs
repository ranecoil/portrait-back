use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier, PasswordHash,
};
use chrono::{DateTime, Utc};
use sqlx::{query_as, FromRow, PgPool};
use uuid::Uuid;

#[derive(FromRow)]
pub struct Creator {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub pfp: Option<String>,
    pub pw_hash: String,
    pub created: DateTime<Utc>
}

impl Creator {
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

    pub async fn get_by_id(id: Uuid, db: &PgPool) -> Result<Self> {
        Ok(
            query_as!(Creator, "SELECT * FROM creators WHERE id = $1", id)
                .fetch_one(db)
                .await?,
        )
    }

    pub async fn get_by_name(name: String, db: &PgPool) -> Result<Self> {
        Ok(
            query_as!(Creator, "SELECT * FROM creators WHERE name = $1", name)
                .fetch_one(db)
                .await?,
        )
    } 

    pub async fn verify_by_name(name: String, password: String, db: &PgPool) -> Result<()> {
        let creator = Self::get_by_name(name, db).await?;
        Argon2::default().verify_password(&password.into_bytes(), &PasswordHash::new(&creator.pw_hash)?)?;
        Ok(())
    }

    pub async fn verify_by_email(email: String, password: String, db: &PgPool) -> Result<()> {
        let creator = Self::get_by_name(email, db).await?;
        Argon2::default().verify_password(&password.into_bytes(), &PasswordHash::new(&creator.pw_hash)?)?;
        Ok(())
    }
}