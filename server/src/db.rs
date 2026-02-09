use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

pub async fn init_pool() -> Result<SqlitePool, sqlx::Error> {
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:aiindex.db".to_string());

    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS prices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            price REAL NOT NULL,
            change REAL,
            change_pct REAL,
            market_cap REAL,
            timestamp TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_prices_symbol_ts ON prices(symbol, timestamp)",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS index_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value REAL NOT NULL,
            daily_change REAL,
            daily_change_pct REAL,
            timestamp TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS base_prices (
            symbol TEXT PRIMARY KEY,
            price REAL NOT NULL,
            recorded_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    tracing::info!("Database initialized");
    Ok(pool)
}
