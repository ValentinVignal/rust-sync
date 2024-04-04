use fakeit::{self, color, words};
use nanoid::nanoid;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

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

pub async fn seed() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://root:password@localhost:5555/test")
        .await
        .expect("Failed to connect to Postgres");

    log::debug!("Seeding database...");
    create_database(&pool).await?;

    for _ in 0..COUNTS.projects {
        sqlx::query(r#"INSERT INTO project (id, name, code) VALUES ($1, $2, $3)"#)
            .bind(nanoid!())
            .bind(words::word())
            .bind(color::safe())
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
