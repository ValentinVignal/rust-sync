use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;

#[derive(Serialize, Deserialize)]
pub struct SyncedProject {
    pub id: String,
    pub name: String,
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct SyncData {
    pub projects: Vec<SyncedProject>,
}

pub async fn sync() -> Result<SyncData, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://root:password@localhost:5555/test")
        .await
        .expect("Failed to connect to Postgres");

    let projects = sqlx::query_as!(SyncedProject, 
        r#"SELECT id, name, code FROM project WHERE "deletedAt" IS NULL ORDER BY "updatedAt" DESC LIMIT 1000"#
    ).fetch_all(&pool).await?;



    Ok(SyncData { projects })
}
