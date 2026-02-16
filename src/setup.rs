// src/setup.rs

use sea_orm::*;

const DATABASE_URL: &str = "postgres://postgres:example@localhost:5432";
const DB_NAME: &str = "soliloquio";

pub(super) async fn set_up_db() -> Result<DatabaseConnection, DbErr> {
    let url = format!("{}/{}", DATABASE_URL, DB_NAME);
    let mut opts = ConnectOptions::new(url);
    opts.sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Debug);

    let db = Database::connect(opts).await?;
    tracing::info!("DB connected");
    Ok(db)
}
