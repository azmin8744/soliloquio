// src/setup.rs

use sea_orm::*;

pub(super) async fn set_up_db() -> Result<DatabaseConnection, DbErr> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut opts = ConnectOptions::new(url);
    opts.sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Debug);

    let db = Database::connect(opts).await?;
    tracing::info!("DB connected");
    Ok(db)
}
