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
    /// Create a new session for a given user.
    pub async fn new(subject: Uuid, db: &PgPool) -> Result<Self, anyhow::Error> {
        let x = query_as!(Session, "INSERT INTO sessions (subject) VALUES ($1) RETURNING *", subject).fetch_one(db).await?;
        Ok(x)
    }

    /// Fetch a session by its token.
    pub async fn get_by_token(token: Uuid, db: &PgPool) -> Result<Option<Self>, anyhow::Error> {
        let x = query_as!(Session, "SELECT * FROM sessions WHERE token = $1", token).fetch_optional(db).await?;
        Ok(x)
    }

    /// Get all currently active sessions by its subject.
    /// 
    /// Note: The `Ok()` result will never have an empty `Some(Vec<Session>)`, but instead return `None`.
    pub async fn get_by_subject(subject: Uuid, db: &PgPool) -> Result<Option<Vec<Self>>, anyhow::Error> {
        let x = query_as!(Session, "SELECT * FROM sessions WHERE subject = $1", subject).fetch_all(db).await?;
        match x.len() {
            0 => Ok(None),
            _ => Ok(Some(x))
        }
    }

    /// Remove a single session.
    /// 
    /// If you just want to remove a session and only got its `token` you're most likely searching for [`Session::remove_by_token()`]
    pub async fn remove(&self, db: &PgPool) -> Result<(), anyhow::Error> {
        query!("DELETE FROM sessions WHERE token = $1", self.token).execute(db).await?;
        Ok(())
    }

    #[doc(alias = "remove")]
    /// Remove a single session by its token.
    pub async fn remove_by_token(token: Uuid, db: &PgPool) -> Result<(), anyhow::Error> {
        query!("DELETE FROM sessions WHERE token = $1", token).execute(db).await?;
        Ok(())
    }


    /// Remove all of the subjects currently known sessions.
    /// 
    /// This will not return the number of affected rows and there's currently no way to do that w\ adding a dedicated function.
    pub async fn remove_by_subject(subject: Uuid, db: &PgPool) -> Result<(), anyhow::Error> {
        query!("DELETE FROM sessions WHERE subject = $1", subject).execute(db).await?;
        Ok(())
    }
}