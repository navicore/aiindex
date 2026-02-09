use crate::config::StocksConfig;
use crate::models::IndexSnapshot;
use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Compute the hybrid-weighted index value and store a snapshot.
pub async fn compute_and_store(
    pool: &SqlitePool,
    config: &Arc<StocksConfig>,
) -> Option<IndexSnapshot> {
    let index_symbols = config.index_symbols();
    let mcap_pct = config.mcap_pct();
    let base_value = config.settings.base_value;
    let n = index_symbols.len() as f64;

    if n == 0.0 {
        return None;
    }

    // Gather latest price + base price + market cap for each index symbol.
    let mut entries: Vec<(f64, f64, f64)> = Vec::new(); // (current, base, mcap)

    for sym in &index_symbols {
        let latest = sqlx::query_as::<_, (f64, Option<f64>)>(
            "SELECT price, market_cap FROM prices
             WHERE symbol = ? ORDER BY timestamp DESC LIMIT 1",
        )
        .bind(sym)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let base = sqlx::query_as::<_, (f64,)>(
            "SELECT price FROM base_prices WHERE symbol = ?",
        )
        .bind(sym)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        if let (Some((current_price, mcap_opt)), Some((base_price,))) = (latest, base) {
            if base_price > 0.0 && current_price > 0.0 {
                let mcap = mcap_opt.unwrap_or(1.0); // fallback equal
                entries.push((current_price, base_price, mcap));
            }
        }
    }

    if entries.is_empty() {
        return None;
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

    // Compute daily change from previous snapshot.
    let prev = sqlx::query_as::<_, (f64,)>(
        "SELECT value FROM index_snapshots ORDER BY timestamp DESC LIMIT 1",
    )
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

    let now = Utc::now().to_rfc3339();

    let _ = sqlx::query(
        "INSERT INTO index_snapshots (value, daily_change, daily_change_pct, timestamp)
         VALUES (?, ?, ?, ?)",
    )
    .bind(index_value)
    .bind(daily_change)
    .bind(daily_change_pct)
    .bind(&now)
    .execute(pool)
    .await;

    let snapshot = IndexSnapshot {
        value: index_value,
        daily_change,
        daily_change_pct,
        timestamp: now,
    };

    tracing::info!("Index computed: {:.2}", index_value);
    Some(snapshot)
}
