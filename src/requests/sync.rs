use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;

#[derive(Serialize, Deserialize)]
pub struct SyncedProject {
    pub id: String,
    pub name: String,
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct SyncedForm {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub data: Value,
    pub task_ids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncedTask {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub project_id: String,
    pub form_ids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncData {
    pub projects: Vec<SyncedProject>,
    pub forms: Vec<SyncedForm>,
    pub tasks: Vec<SyncedTask>,
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

    let forms = sqlx::query_as!(SyncedForm,
        r###"SELECT form.id, form.name, form."projectID" as project_id, form.data, array_remove(array_agg(forms_tasks."taskID"), NULL) as task_ids FROM form
        LEFT JOIN forms_tasks ON form.id = forms_tasks."formID"
        WHERE form."deletedAt" IS NULL
        GROUP BY form.id
        ORDER BY form."updatedAt" DESC LIMIT 1000;"###
    ).fetch_all(&pool).await?;


    let tasks = sqlx::query_as!(SyncedTask,
        r###"SELECT task.id, task.name, task.description, task."projectID" as project_id, array_remove(array_agg(forms_tasks."formID"), NULL) as form_ids FROM task
        LEFT JOIN forms_tasks ON task.id = forms_tasks."taskID"
        WHERE task."deletedAt" IS NULL
        GROUP BY task.id
        ORDER BY task."updatedAt" DESC LIMIT 1000"###
    ).fetch_all(&pool).await?;


    Ok(SyncData { projects, forms, tasks })
}
