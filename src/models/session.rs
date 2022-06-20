use chrono::{DateTime, Utc};
use sqlx::{FromRow, query_as, PgPool, query};
use uuid::Uuid;


#[derive(FromRow)]
pub struct Session {
    pub token: Uuid,
    pub subject: Uuid,
    pub created: DateTime<Utc>
}

impl Session {
    pub async fn new(subject: Uuid, db: &PgPool) -> Result<Self, anyhow::Error> {
        let x = query_as!(Session, "INSERT INTO sessions (subject) VALUES ($1) RETURNING *", subject).fetch_one(db).await?;
        Ok(x)
    }

    pub async fn get_by_token(token: Uuid, db: &PgPool) -> Result<Option<Self>, anyhow::Error> {
        let x = query_as!(Session, "SELECT * FROM sessions WHERE token = $1", token).fetch_optional(db).await?;
        Ok(x)
    }

    pub async fn get_by_subject(subject: Uuid, db: &PgPool) -> Result<Option<Vec<Self>>, anyhow::Error> {
        let x = query_as!(Session, "SELECT * FROM sessions WHERE subject = $1", subject).fetch_all(db).await?;
        match x.len() {
            0 => Ok(None),
            _ => Ok(Some(x))
        }
    }

    pub async fn remove(&self, db: &PgPool) -> Result<(), anyhow::Error> {
        query!("DELETE FROM sessions WHERE token = $1", self.token).execute(db).await?;
        Ok(())
    }

    pub async fn remove_by_token(token: Uuid, db: &PgPool) -> Result<(), anyhow::Error> {
        query!("DELETE FROM sessions WHERE token = $1", token).execute(db).await?;
        Ok(())
    }

    
    pub async fn remove_by_subject(subject: Uuid, db: &PgPool) -> Result<(), anyhow::Error> {
        query!("DELETE FROM sessions WHERE subject = $1", subject).execute(db).await?;
        Ok(())
    }
}