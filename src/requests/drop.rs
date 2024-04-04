use sqlx::postgres::PgPoolOptions;

pub async fn drop() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://root:password@localhost:5555/test")
        .await
        .expect("Failed to connect to Postgres");

    log::debug!("Dropping database...");
    // TODO: Don't create the db if it already exists.

    let drop_queries = [
        r#"ALTER TABLE "form_projects_project" DROP CONSTRAINT "FK_99033b0d627d82d697e1b3b08bf""#,
        r#"ALTER TABLE "form_projects_project" DROP CONSTRAINT "FK_bc419c142f5336f4f3c4849788f""#,
        r#"ALTER TABLE "form" DROP CONSTRAINT "FK_793836ec378a587c98a8c72a6b8""#,
        r#"ALTER TABLE "forms_tasks" DROP CONSTRAINT "FK_0bc7355812c3784dd05b38e13f6""#,
        r#"ALTER TABLE "forms_tasks" DROP CONSTRAINT "FK_f3ed34ef693480eda462df17b7b""#,
        r#"ALTER TABLE "task" DROP CONSTRAINT "FK_464e1e9f04be8ced7e4e878fbcf""#,
        r#"DROP INDEX "public"."IDX_99033b0d627d82d697e1b3b08b""#,
        r#"DROP INDEX "public"."IDX_bc419c142f5336f4f3c4849788""#,
        r#"DROP TABLE "form_projects_project""#,
        r#"DROP TABLE "form""#,
        r#"DROP TABLE "forms_tasks""#,
        r#"DROP TABLE "task""#,
        r#"DROP TABLE "project""#,
    ];

    for query in drop_queries {
        sqlx::query(query).execute(&pool).await?;
    }

    log::debug!("Done dropping");
    Ok(())
}
