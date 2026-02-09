use crate::config::StocksConfig;
use crate::index;
use crate::models::*;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Arc<StocksConfig>,
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn get_index(State(state): State<AppState>) -> Json<serde_json::Value> {
    let snapshot = index::compute_and_store(&state.pool, &state.config).await;
    match snapshot {
        Some(s) => Json(serde_json::json!(s)),
        None => Json(serde_json::json!({
            "value": null,
            "message": "No data available yet"
        })),
    }
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
}

pub async fn get_index_history(
    State(state): State<AppState>,
    Query(q): Query<HistoryQuery>,
) -> Json<Vec<IndexSnapshot>> {
    let limit = q.limit.unwrap_or(100);
    let rows = sqlx::query_as::<_, (f64, Option<f64>, Option<f64>, String)>(
        "SELECT value, daily_change, daily_change_pct, timestamp
         FROM index_snapshots ORDER BY timestamp DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let snapshots: Vec<IndexSnapshot> = rows
        .into_iter()
        .map(
            |(value, daily_change, daily_change_pct, timestamp)| IndexSnapshot {
                value,
                daily_change,
                daily_change_pct,
                timestamp,
            },
        )
        .collect();

    Json(snapshots)
}

pub async fn get_stocks(State(state): State<AppState>) -> Json<Vec<StockDetail>> {
    let mut stocks = Vec::new();

    // Compute weights for display.
    let weights = compute_weights(&state.pool, &state.config).await;

    for (sector_key, sector) in &state.config.sectors {
        for sym in &sector.symbols {
            let latest = sqlx::query_as::<_, (f64, Option<f64>, Option<f64>, Option<f64>, String)>(
                "SELECT price, change, change_pct, market_cap, timestamp
                 FROM prices WHERE symbol = ? ORDER BY timestamp DESC LIMIT 1",
            )
            .bind(sym)
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten();

            if let Some((price, change, change_pct, market_cap, timestamp)) = latest {
                stocks.push(StockDetail {
                    symbol: sym.clone(),
                    sector: sector_key.clone(),
                    sector_label: sector.label.clone(),
                    price,
                    change,
                    change_pct,
                    market_cap,
                    weight: weights.get(sym).copied(),
                    timestamp,
                });
            }
        }
    }

    // Add benchmarks.
    for sym in &state.config.benchmarks.symbols {
        let latest = sqlx::query_as::<_, (f64, Option<f64>, Option<f64>, Option<f64>, String)>(
            "SELECT price, change, change_pct, market_cap, timestamp
             FROM prices WHERE symbol = ? ORDER BY timestamp DESC LIMIT 1",
        )
        .bind(sym)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten();

        if let Some((price, change, change_pct, market_cap, timestamp)) = latest {
            stocks.push(StockDetail {
                symbol: sym.clone(),
                sector: "benchmarks".to_string(),
                sector_label: "Benchmarks".to_string(),
                price,
                change,
                change_pct,
                market_cap,
                weight: None,
                timestamp,
            });
        }
    }

    Json(stocks)
}

pub async fn get_stock(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<StockDetail>, StatusCode> {
    let sym = symbol.to_uppercase();

    // Find sector for this symbol.
    let (sector_key, sector_label) = state
        .config
        .sectors
        .iter()
        .find(|(_, s)| s.symbols.contains(&sym))
        .map(|(k, s)| (k.clone(), s.label.clone()))
        .unwrap_or_else(|| {
            if state.config.benchmarks.symbols.contains(&sym) {
                ("benchmarks".to_string(), "Benchmarks".to_string())
            } else {
                ("unknown".to_string(), "Unknown".to_string())
            }
        });

    let latest = sqlx::query_as::<_, (f64, Option<f64>, Option<f64>, Option<f64>, String)>(
        "SELECT price, change, change_pct, market_cap, timestamp
         FROM prices WHERE symbol = ? ORDER BY timestamp DESC LIMIT 1",
    )
    .bind(&sym)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let weights = compute_weights(&state.pool, &state.config).await;
    let (price, change, change_pct, market_cap, timestamp) = latest;

    Ok(Json(StockDetail {
        symbol: sym.clone(),
        sector: sector_key,
        sector_label,
        price,
        change,
        change_pct,
        market_cap,
        weight: weights.get(&sym).copied(),
        timestamp,
    }))
}

pub async fn get_sectors(State(state): State<AppState>) -> Json<Vec<SectorSummary>> {
    let weights = compute_weights(&state.pool, &state.config).await;
    let mut sectors = Vec::new();

    for (key, sector) in &state.config.sectors {
        let total_weight: f64 = sector.symbols.iter().filter_map(|s| weights.get(s)).sum();

        let mut changes = Vec::new();
        for sym in &sector.symbols {
            let change_pct = sqlx::query_as::<_, (Option<f64>,)>(
                "SELECT change_pct FROM prices WHERE symbol = ? ORDER BY timestamp DESC LIMIT 1",
            )
            .bind(sym)
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten()
            .and_then(|(c,)| c);

            if let Some(c) = change_pct {
                changes.push(c);
            }
        }

        let avg_change_pct = if changes.is_empty() {
            0.0
        } else {
            changes.iter().sum::<f64>() / changes.len() as f64
        };

        sectors.push(SectorSummary {
            key: key.clone(),
            label: sector.label.clone(),
            symbols: sector.symbols.clone(),
            total_weight,
            avg_change_pct,
        });
    }

    Json(sectors)
}

pub async fn get_config(State(state): State<AppState>) -> Json<ConfigInfo> {
    Json(ConfigInfo {
        base_value: state.config.settings.base_value,
        market_cap_weight_pct: state.config.settings.market_cap_weight_pct,
        index_stock_count: state.config.index_symbols().len(),
        benchmark_symbols: state.config.benchmarks.symbols.clone(),
    })
}

/// Compute blended weights for all index symbols.
async fn compute_weights(
    pool: &SqlitePool,
    config: &StocksConfig,
) -> std::collections::HashMap<String, f64> {
    let index_symbols = config.index_symbols();
    let mcap_pct = config.mcap_pct();
    let n = index_symbols.len() as f64;
    let equal_weight = if n > 0.0 { 1.0 / n } else { 0.0 };

    let mut mcaps: Vec<(String, f64)> = Vec::new();

    for sym in &index_symbols {
        let mcap = sqlx::query_as::<_, (Option<f64>,)>(
            "SELECT market_cap FROM prices WHERE symbol = ? ORDER BY timestamp DESC LIMIT 1",
        )
        .bind(sym)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .and_then(|(m,)| m)
        .unwrap_or(1.0);

        mcaps.push((sym.clone(), mcap));
    }

    let total_mcap: f64 = mcaps.iter().map(|(_, m)| m).sum();

    let mut weights = std::collections::HashMap::new();
    for (sym, mcap) in &mcaps {
        let mcap_weight = if total_mcap > 0.0 {
            mcap / total_mcap
        } else {
            equal_weight
        };
        let blended = (mcap_pct * mcap_weight) + ((1.0 - mcap_pct) * equal_weight);
        weights.insert(sym.clone(), blended);
    }

    weights
}
