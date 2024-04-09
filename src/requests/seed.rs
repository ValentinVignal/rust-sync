use fakeit::{self, color, datetime, generator, hipster, password, words};
use nanoid::nanoid;
use rand::seq::SliceRandom;

use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
// cspell:ignore unnest, tablename, datetime

struct Counts {
    projects: i32,
    forms: i32,
    tasks: i32,
    form_to_task: i32,
}

const COUNTS: Counts = Counts {
    projects: 500,
    forms: 50_000,
    tasks: 50_000,
    form_to_task: 10_000,
};

const BATCH_SIZE: i32 = 100;

pub async fn seed() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://root:password@localhost:5555/test")
        .await
        .expect("Failed to connect to Postgres");

    log::debug!("Seeding database...");
    create_database(&pool).await?;

    for _ in 0..COUNTS.projects / BATCH_SIZE {
        let mut project_ids: Vec<String> = Vec::new();
        let mut project_names: Vec<String> = Vec::new();
        let mut project_codes: Vec<String> = Vec::new();
        for _ in 0..BATCH_SIZE {
            project_ids.push(nanoid!());
            project_names.push(words::word());
            project_codes.push(color::safe());
        }

        sqlx::query(r#"INSERT INTO project (id, name, code) SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])"#)
            .bind(&project_ids[..])
            .bind(&project_names[..])
            .bind(&project_codes[..])
            .execute(&pool)
            .await?;
    }

    let project_ids = sqlx::query_as::<_, (String,)>("SELECT id FROM project")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|(row,)| row)
        .collect::<Vec<String>>();

    let field_ids: Vec<&str> = "0123456789abcdefghijklmnopqrstuvwxyz"
        .split("")
        .collect::<Vec<&str>>();

    for _ in 0..COUNTS.forms / BATCH_SIZE {
        let mut form_ids: Vec<String> = Vec::new();
        let mut form_names: Vec<String> = Vec::new();
        let mut form_project_ids: Vec<String> = Vec::new();
        let mut form_data: Vec<Value> = Vec::new();
        let mut form_updated_ats: Vec<i64> = Vec::new();
        for _ in 0..BATCH_SIZE {
            form_ids.push(nanoid!());
            form_names.push(password::generate(true, true, false, 10));
            form_project_ids.push(project_ids.choose(&mut rand::thread_rng()).unwrap().clone());
            let mut data = json!({});
            for id in field_ids.iter() {
                data[id] = Value::String(words::sentence(10));
            }
            form_data.push(data);
            form_updated_ats.push(datetime::date().secs * 1000);
        }

        sqlx::query(r#"INSERT INTO form (id, name, "projectID", data, "updatedAt") SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::jsonb[], $5::bigint[])"#)
            .bind(&form_ids[..])
            .bind(&form_names[..])
            .bind(&form_project_ids[..])
            .bind(&form_data[..])
            .bind(&form_updated_ats[..])
            .execute(&pool)
            .await?;
    }

    for _ in 0..COUNTS.tasks / BATCH_SIZE {
        let mut task_ids: Vec<String> = Vec::new();
        let mut task_names: Vec<String> = Vec::new();
        let mut task_project_ids: Vec<String> = Vec::new();
        let mut task_descriptions: Vec<String> = Vec::new();
        let mut task_updated_ats: Vec<i64> = Vec::new();
        for _ in 0..BATCH_SIZE {
            task_ids.push(nanoid!());
            task_names.push(generator::generate(
                "{hacker.verb} {hacker.noun}".to_string(),
            ));
            task_project_ids.push(project_ids.choose(&mut rand::thread_rng()).unwrap().clone());
            task_descriptions.push(hipster::paragraph(3, 4, 40, " ".to_string()));
            task_updated_ats.push(datetime::date().secs * 1000);
        }

        sqlx::query(r#"INSERT INTO task (id, name, "projectID", description, "updatedAt") SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::bigint[])"#)
            .bind(&task_ids[..])
            .bind(&task_names[..])
            .bind(&task_project_ids[..])
            .bind(&task_descriptions[..])
            .bind(&task_updated_ats[..])
            .execute(&pool)
            .await?;
    }

    let task_ids = sqlx::query_as::<_, (String,)>("SELECT id FROM task")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|(row,)| row)
        .collect::<Vec<String>>();
    let form_ids = sqlx::query_as::<_, (String,)>("SELECT id FROM form")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|(row,)| row)
        .collect::<Vec<String>>();

    for _ in 0..COUNTS.form_to_task / BATCH_SIZE {
        let mut form_ids_batch = Vec::<String>::new();
        let mut task_ids_batch = Vec::<String>::new();
        let mut updated_ats = Vec::<i64>::new();
        for _ in 0..BATCH_SIZE {
            form_ids_batch.push(form_ids.choose(&mut rand::thread_rng()).unwrap().clone());
            task_ids_batch.push(task_ids.choose(&mut rand::thread_rng()).unwrap().clone());
            updated_ats.push(datetime::date().secs * 1000);
        }
        sqlx::query(r#"INSERT INTO forms_tasks ("formID", "taskID", "updatedAt") SELECT * FROM UNNEST($1::text[], $2::text[], $3::bigint[])"#)
            .bind(&form_ids_batch[..])
            .bind(&task_ids_batch[..])
            .bind(&updated_ats[..])
            .execute(&pool)
            .await?;
    }

    log::debug!("Done seeding");
    Ok(())
}

async fn create_database(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    let tables_exist: (bool,)= sqlx::query_as(  "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename  = 'project')").fetch_one(pool).await?;
    if !tables_exist.0 {
        log::debug!("Creating tables...");
        let create_queries = [
            r#"CREATE TABLE "project" ("updatedAt" bigint NOT NULL DEFAULT '0', "deletedAt" TIMESTAMP, "id" character varying NOT NULL, "name" character varying NOT NULL, "code" character varying NOT NULL, CONSTRAINT "PK_4d68b1358bb5b766d3e78f32f57" PRIMARY KEY ("id"))"#,
            r#"CREATE TABLE "task" ("updatedAt" bigint NOT NULL DEFAULT '0', "deletedAt" TIMESTAMP, "id" character varying NOT NULL, "projectID" character varying NOT NULL, "name" character varying NOT NULL, "description" character varying, CONSTRAINT "PK_fb213f79ee45060ba925ecd576e" PRIMARY KEY ("id"))"#,
            r#"CREATE TABLE "forms_tasks" ("updatedAt" bigint NOT NULL DEFAULT '0', "deletedAt" TIMESTAMP, "formID" character varying NOT NULL, "taskID" character varying NOT NULL, CONSTRAINT "PK_5cde27784334db1c9530bea6b5f" PRIMARY KEY ("formID", "taskID"))"#,
            r#"CREATE TABLE "form" ("updatedAt" bigint NOT NULL DEFAULT '0', "deletedAt" TIMESTAMP, "id" character varying NOT NULL, "projectID" character varying NOT NULL, "name" character varying NOT NULL, "data" jsonb NOT NULL DEFAULT '{}', CONSTRAINT "PK_8f72b95aa2f8ba82cf95dc7579e" PRIMARY KEY ("id"))"#,
            r#"CREATE TABLE "form_projects_project" ("formId" character varying NOT NULL, "projectId" character varying NOT NULL, CONSTRAINT "PK_0db033acf146ce2e7f99433877a" PRIMARY KEY ("formId", "projectId"))"#,
            r#"CREATE INDEX "IDX_bc419c142f5336f4f3c4849788" ON "form_projects_project" ("formId") "#,
            r#"CREATE INDEX "IDX_99033b0d627d82d697e1b3b08b" ON "form_projects_project" ("projectId") "#,
            // cspell: disable-next-line
            r#"ALTER TABLE "task" ADD CONSTRAINT "FK_464e1e9f04be8ced7e4e878fbcf" FOREIGN KEY ("projectID") REFERENCES "project"("id") ON DELETE NO ACTION ON UPDATE NO ACTION"#,
            r#"ALTER TABLE "forms_tasks" ADD CONSTRAINT "FK_f3ed34ef693480eda462df17b7b" FOREIGN KEY ("formID") REFERENCES "form"("id") ON DELETE CASCADE ON UPDATE NO ACTION"#,
            r#"ALTER TABLE "forms_tasks" ADD CONSTRAINT "FK_0bc7355812c3784dd05b38e13f6" FOREIGN KEY ("taskID") REFERENCES "task"("id") ON DELETE CASCADE ON UPDATE NO ACTION"#,
            r#"ALTER TABLE "form" ADD CONSTRAINT "FK_793836ec378a587c98a8c72a6b8" FOREIGN KEY ("projectID") REFERENCES "project"("id") ON DELETE NO ACTION ON UPDATE NO ACTION"#,
            r#"ALTER TABLE "form_projects_project" ADD CONSTRAINT "FK_bc419c142f5336f4f3c4849788f" FOREIGN KEY ("formId") REFERENCES "form"("id") ON DELETE CASCADE ON UPDATE CASCADE"#,
            r#"ALTER TABLE "form_projects_project" ADD CONSTRAINT "FK_99033b0d627d82d697e1b3b08bf" FOREIGN KEY ("projectId") REFERENCES "project"("id") ON DELETE CASCADE ON UPDATE CASCADE"#,
        ];

        for query in create_queries {
            sqlx::query(query).execute(pool).await?;
        }
        log::debug!("Done creating tables");
    } else {
        log::debug!("Tables already created");
    }
    Ok(())
}
