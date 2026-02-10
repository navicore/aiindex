use crate::config::StocksConfig;
use crate::index;
use crate::models::{FinnhubCandle, FinnhubProfile, FinnhubQuote};
use chrono::{DateTime, Utc};
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
        // Daily profile refresh (the first fetch happens in quote_loop before quotes).
        time::sleep(PROFILE_INTERVAL).await;
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

    // On first run, fetch profiles before quotes so market_cap is available.
    fetch_all_profiles(&client, &api_key, &pool, &config).await;

    // Backfill historical data if the database is fresh.
    backfill_history(&client, &api_key, &pool, &config).await;

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
                // Carry forward last known market_cap.
                let prev_mcap = sqlx::query_as::<_, (Option<f64>,)>(
                    "SELECT market_cap FROM prices WHERE symbol = ? AND market_cap IS NOT NULL ORDER BY timestamp DESC LIMIT 1",
                )
                .bind(symbol)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .and_then(|(m,)| m);

                if let Err(e) = sqlx::query(
                    "INSERT INTO prices (symbol, price, change, change_pct, market_cap, timestamp)
                     VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(symbol)
                .bind(q.c)
                .bind(q.d)
                .bind(q.dp)
                .bind(prev_mcap)
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

    let now = Utc::now().to_rfc3339();
    for symbol in &symbols {
        match fetch_profile(client, api_key, symbol).await {
            Ok(p) => {
                if let Some(mcap) = p.market_capitalization {
                    // Update the most recent price row for this symbol with market cap.
                    let _ = sqlx::query(
                        "UPDATE prices SET market_cap = ?
                         WHERE id = (SELECT id FROM prices WHERE symbol = ? ORDER BY timestamp DESC LIMIT 1)",
                    )
                    .bind(mcap)
                    .bind(symbol)
                    .execute(pool)
                    .await;
                }
                // Store profile info.
                let _ = sqlx::query(
                    "INSERT INTO stock_profiles (symbol, name, exchange, industry, weburl, logo, country, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(symbol) DO UPDATE SET
                       name = excluded.name,
                       exchange = excluded.exchange,
                       industry = excluded.industry,
                       weburl = excluded.weburl,
                       logo = excluded.logo,
                       country = excluded.country,
                       updated_at = excluded.updated_at",
                )
                .bind(symbol)
                .bind(&p.name)
                .bind(&p.exchange)
                .bind(&p.finnhub_industry)
                .bind(&p.weburl)
                .bind(&p.logo)
                .bind(&p.country)
                .bind(&now)
                .execute(pool)
                .await;
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

async fn fetch_candle(
    client: &reqwest::Client,
    api_key: &str,
    symbol: &str,
    from: i64,
    to: i64,
) -> Result<FinnhubCandle, reqwest::Error> {
    let url = format!(
        "https://finnhub.io/api/v1/stock/candle?symbol={}&resolution=D&from={}&to={}&token={}",
        symbol, from, to, api_key
    );
    client.get(&url).send().await?.json::<FinnhubCandle>().await
}

/// Backfill ~1 year of daily history if the database has no historical data.
async fn backfill_history(
    client: &reqwest::Client,
    api_key: &str,
    pool: &SqlitePool,
    config: &StocksConfig,
) {
    // Check if we already have substantial data.
    let count = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM prices")
        .fetch_one(pool)
        .await
        .map(|(c,)| c)
        .unwrap_or(0);

    if count > 100 {
        tracing::info!("Database has {} price rows, skipping backfill", count);
        return;
    }

    tracing::info!("Backfilling historical data...");

    let now = Utc::now();
    let one_year_ago = now - chrono::Duration::days(365);
    let from_ts = one_year_ago.timestamp();
    let to_ts = now.timestamp();

    let symbols = config.all_symbols();

    // Fetch candles for all symbols.
    for symbol in &symbols {
        match fetch_candle(client, api_key, symbol, from_ts, to_ts).await {
            Ok(candle) => {
                if candle.s != "ok" {
                    tracing::warn!("{}: no candle data (status={})", symbol, candle.s);
                    continue;
                }
                let closes = candle.c.unwrap_or_default();
                let timestamps = candle.t.unwrap_or_default();

                if closes.is_empty() {
                    continue;
                }

                // Set base price from earliest close.
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO base_prices (symbol, price, recorded_at)
                     VALUES (?, ?, ?)",
                )
                .bind(symbol)
                .bind(closes[0])
                .bind(
                    DateTime::from_timestamp(timestamps[0], 0)
                        .unwrap_or(now)
                        .to_rfc3339(),
                )
                .execute(pool)
                .await;

                // Look up current market_cap for this symbol to attach to rows.
                let mcap = sqlx::query_as::<_, (Option<f64>,)>(
                    "SELECT market_cap FROM prices WHERE symbol = ? AND market_cap IS NOT NULL ORDER BY timestamp DESC LIMIT 1",
                )
                .bind(symbol)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .and_then(|(m,)| m);

                // Insert daily closes.
                for (i, (price, ts)) in closes.iter().zip(timestamps.iter()).enumerate() {
                    let dt = DateTime::from_timestamp(*ts, 0).unwrap_or(now).to_rfc3339();

                    // Compute daily change from previous close.
                    let (change, change_pct) = if i > 0 && closes[i - 1] > 0.0 {
                        let chg = price - closes[i - 1];
                        let pct = (chg / closes[i - 1]) * 100.0;
                        (Some(chg), Some(pct))
                    } else {
                        (None, None)
                    };

                    let _ = sqlx::query(
                        "INSERT INTO prices (symbol, price, change, change_pct, market_cap, timestamp)
                         VALUES (?, ?, ?, ?, ?, ?)",
                    )
                    .bind(symbol)
                    .bind(price)
                    .bind(change)
                    .bind(change_pct)
                    .bind(mcap)
                    .bind(&dt)
                    .execute(pool)
                    .await;
                }
                tracing::info!("{}: backfilled {} daily candles", symbol, closes.len());
            }
            Err(e) => {
                tracing::error!("{}: candle fetch failed: {}", symbol, e);
            }
        }
        time::sleep(CALL_SPACING).await;
    }

    // Compute historical index snapshots from backfilled data.
    tracing::info!("Computing historical index snapshots...");
    let dates = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT DATE(timestamp) as d FROM prices ORDER BY d",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (date_str,) in &dates {
        // For each date, compute the index from that day's closing prices.
        let index_symbols = config.index_symbols();
        let mcap_pct = config.mcap_pct();
        let base_value = config.settings.base_value;
        let n = index_symbols.len() as f64;
        if n == 0.0 {
            continue;
        }

        let mut entries: Vec<(f64, f64, f64)> = Vec::new();

        for sym in &index_symbols {
            let latest = sqlx::query_as::<_, (f64, Option<f64>)>(
                "SELECT price, market_cap FROM prices
                 WHERE symbol = ? AND DATE(timestamp) <= ?
                 ORDER BY timestamp DESC LIMIT 1",
            )
            .bind(sym)
            .bind(date_str)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

            let base =
                sqlx::query_as::<_, (f64,)>("SELECT price FROM base_prices WHERE symbol = ?")
                    .bind(sym)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten();

            if let (Some((current_price, mcap_opt)), Some((base_price,))) = (latest, base) {
                if base_price > 0.0 && current_price > 0.0 {
                    let mcap = mcap_opt.unwrap_or(1.0);
                    entries.push((current_price, base_price, mcap));
                }
            }
        }

        if entries.is_empty() {
            continue;
        }

        let total_mcap: f64 = entries.iter().map(|(_, _, m)| m).sum();
        let equal_weight = 1.0 / entries.len() as f64;

        let mut index_value = 0.0;
        for (current, base, mcap) in &entries {
            let mcap_weight = if total_mcap > 0.0 {
                mcap / total_mcap
            } else {
                equal_weight
            };
            let blended = (mcap_pct * mcap_weight) + ((1.0 - mcap_pct) * equal_weight);
            index_value += blended * (current / base);
        }
        index_value *= base_value;

        // Get previous snapshot for daily change.
        let prev = sqlx::query_as::<_, (f64,)>(
            "SELECT value FROM index_snapshots WHERE DATE(timestamp) < ? ORDER BY timestamp DESC LIMIT 1",
        )
        .bind(date_str)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let (daily_change, daily_change_pct) = if let Some((prev_val,)) = prev {
            if prev_val > 0.0 {
                let chg = index_value - prev_val;
                let pct = (chg / prev_val) * 100.0;
                (Some(chg), Some(pct))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Use end-of-day timestamp.
        let snap_ts = format!("{}T16:00:00+00:00", date_str);

        let _ = sqlx::query(
            "INSERT INTO index_snapshots (value, daily_change, daily_change_pct, timestamp)
             VALUES (?, ?, ?, ?)",
        )
        .bind(index_value)
        .bind(daily_change)
        .bind(daily_change_pct)
        .bind(&snap_ts)
        .execute(pool)
        .await;
    }

    tracing::info!("Backfill complete: {} trading days", dates.len());
}
