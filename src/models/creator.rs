use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use chrono::{DateTime, Utc};
use hyper::body::Body;
use sqlx::{query_as, FromRow, PgPool};
use uuid::Uuid;

fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

#[derive(Debug, FromRow)]
pub struct Creator {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub picture: Option<Uuid>,
    pub created: DateTime<Utc>,
}

impl Creator {
    /// Create a new creator
    pub async fn new(name: &str, email: &str, password: &str, db: &PgPool) -> anyhow::Result<Self> {
        let password_hash = hash_password(password)?;

        let creator = query_as!(
            Creator,
            "INSERT INTO creators (name, email, password) VALUES ($1, $2, $3) RETURNING *",
            name,
            email,
            password_hash
        )
        .fetch_one(db)
        .await?;

        Ok(creator)
    }

    /// Get a single creator by their id
    pub async fn get(id: &Uuid, db: &PgPool) -> anyhow::Result<Self> {
        let creator = query_as!(Creator, "SELECT * FROM creators WHERE id = $1", id)
            .fetch_one(db)
            .await?;

        Ok(creator)
    }

    /// Lookup a creator by their name.
    pub async fn get_by_name(name: &str, db: &PgPool) -> anyhow::Result<Self> {
        let creator = query_as!(Creator, "SELECT * FROM creators WHERE name = $1", name)
            .fetch_one(db)
            .await?;

        Ok(creator)
    }

    pub fn verify(&self, password: &str) -> anyhow::Result<()> {
        let argon2 = Argon2::default();
        let password_hash = PasswordHash::new(&self.password)?;
        argon2.verify_password(password.as_bytes(), &password_hash)?;
        Ok(())
    }

    pub async fn update(
        id: &Uuid,
        email: Option<&str>,
        password: Option<&str>,
        picture: Option<&Body>,
        db: &PgPool,
    ) -> anyhow::Result<Creator> {
        password.map(|p| hash_password(p)).transpose()?;

        let creator = query_as!(
                Creator,
                "UPDATE creators SET email = COALESCE($1, name), password = COALESCE($2, password) WHERE id = $3 RETURNING *",
                email,
                password,
                id
            )
            .fetch_one(db)
            .await?;

        Ok(creator)
    }
}
