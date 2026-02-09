use crate::config::StocksConfig;
use crate::index;
use crate::models::{FinnhubProfile, FinnhubQuote};
use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::time::{self, Duration};

const QUOTE_INTERVAL: Duration = Duration::from_secs(15 * 60); // 15 minutes
const PROFILE_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours
const CALL_SPACING: Duration = Duration::from_millis(50);

pub fn spawn(pool: SqlitePool, config: Arc<StocksConfig>) {
    let pool_q = pool.clone();
    let config_q = config.clone();
    tokio::spawn(async move {
        quote_loop(pool_q, config_q).await;
    });

    tokio::spawn(async move {
        profile_loop(pool, config).await;
    });
}

async fn quote_loop(pool: SqlitePool, config: Arc<StocksConfig>) {
    let client = reqwest::Client::new();
    let api_key = std::env::var("FINNHUB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        tracing::warn!("FINNHUB_API_KEY not set — fetcher will not run");
        return;
    }

    // Fetch immediately on startup, then every interval.
    loop {
        fetch_all_quotes(&client, &api_key, &pool, &config).await;
        index::compute_and_store(&pool, &config).await;
        time::sleep(QUOTE_INTERVAL).await;
    }
}

async fn profile_loop(pool: SqlitePool, config: Arc<StocksConfig>) {
    let client = reqwest::Client::new();
    let api_key = std::env::var("FINNHUB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return;
    }

    loop {
        fetch_all_profiles(&client, &api_key, &pool, &config).await;
        time::sleep(PROFILE_INTERVAL).await;
    }
}

async fn fetch_all_quotes(
    client: &reqwest::Client,
    api_key: &str,
    pool: &SqlitePool,
    config: &StocksConfig,
) {
    let symbols = config.all_symbols();
    let now = Utc::now().to_rfc3339();
    tracing::info!("Fetching quotes for {} symbols", symbols.len());

    for symbol in &symbols {
        match fetch_quote(client, api_key, symbol).await {
            Ok(q) => {
                if q.c <= 0.0 {
                    tracing::warn!("{}: price is zero, skipping", symbol);
                    continue;
                }
                if let Err(e) = sqlx::query(
                    "INSERT INTO prices (symbol, price, change, change_pct, timestamp)
                     VALUES (?, ?, ?, ?, ?)",
                )
                .bind(symbol)
                .bind(q.c)
                .bind(q.d)
                .bind(q.dp)
                .bind(&now)
                .execute(pool)
                .await
                {
                    tracing::error!("{}: failed to insert price: {}", symbol, e);
                }

                // Record base price if not yet set.
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO base_prices (symbol, price, recorded_at)
                     VALUES (?, ?, ?)",
                )
                .bind(symbol)
                .bind(q.c)
                .bind(&now)
                .execute(pool)
                .await;
            }
            Err(e) => {
                tracing::error!("{}: quote fetch failed: {}", symbol, e);
            }
        }
        time::sleep(CALL_SPACING).await;
    }

    tracing::info!("Quote fetch cycle complete");
}

async fn fetch_all_profiles(
    client: &reqwest::Client,
    api_key: &str,
    pool: &SqlitePool,
    config: &StocksConfig,
) {
    let symbols = config.all_symbols();
    tracing::info!("Fetching profiles for {} symbols", symbols.len());

    for symbol in &symbols {
        match fetch_profile(client, api_key, symbol).await {
            Ok(p) => {
                if let Some(mcap) = p.market_capitalization {
                    // Update the most recent price row for this symbol with market cap.
                    if let Err(e) = sqlx::query(
                        "UPDATE prices SET market_cap = ?
                         WHERE id = (SELECT id FROM prices WHERE symbol = ? ORDER BY timestamp DESC LIMIT 1)",
                    )
                    .bind(mcap)
                    .bind(symbol)
                    .execute(pool)
                    .await
                    {
                        tracing::error!("{}: failed to update market_cap: {}", symbol, e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("{}: profile fetch failed: {}", symbol, e);
            }
        }
        time::sleep(CALL_SPACING).await;
    }

    tracing::info!("Profile fetch cycle complete");
}

async fn fetch_quote(
    client: &reqwest::Client,
    api_key: &str,
    symbol: &str,
) -> Result<FinnhubQuote, reqwest::Error> {
    let url = format!(
        "https://finnhub.io/api/v1/quote?symbol={}&token={}",
        symbol, api_key
    );
    client.get(&url).send().await?.json::<FinnhubQuote>().await
}

async fn fetch_profile(
    client: &reqwest::Client,
    api_key: &str,
    symbol: &str,
) -> Result<FinnhubProfile, reqwest::Error> {
    let url = format!(
        "https://finnhub.io/api/v1/stock/profile2?symbol={}&token={}",
        symbol, api_key
    );
    client
        .get(&url)
        .send()
        .await?
        .json::<FinnhubProfile>()
        .await
}
