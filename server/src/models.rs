use serde::{Deserialize, Serialize};

/// Finnhub quote response.
#[derive(Debug, Deserialize)]
pub struct FinnhubQuote {
    /// Current price
    pub c: f64,
    /// Change
    pub d: Option<f64>,
    /// Percent change
    pub dp: Option<f64>,
}

/// Finnhub company profile response.
#[derive(Debug, Deserialize)]
pub struct FinnhubProfile {
    pub name: Option<String>,
    pub exchange: Option<String>,
    #[serde(rename = "finnhubIndustry")]
    pub finnhub_industry: Option<String>,
    pub weburl: Option<String>,
    pub logo: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "marketCapitalization")]
    pub market_capitalization: Option<f64>,
}

/// A price record stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PriceRecord {
    pub symbol: String,
    pub price: f64,
    pub change: Option<f64>,
    pub change_pct: Option<f64>,
    pub market_cap: Option<f64>,
    pub timestamp: String,
}

/// A computed index snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSnapshot {
    pub value: f64,
    pub daily_change: Option<f64>,
    pub daily_change_pct: Option<f64>,
    pub timestamp: String,
}

/// Base price for a stock, used in index calculation.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct BasePrice {
    pub symbol: String,
    pub price: f64,
    pub recorded_at: String,
}

/// Stock detail returned by the API.
#[derive(Debug, Serialize)]
pub struct StockDetail {
    pub symbol: String,
    pub sector: String,
    pub sector_label: String,
    pub price: f64,
    pub change: Option<f64>,
    pub change_pct: Option<f64>,
    pub market_cap: Option<f64>,
    pub weight: Option<f64>,
    pub timestamp: String,
    pub name: Option<String>,
    pub exchange: Option<String>,
    pub industry: Option<String>,
    pub weburl: Option<String>,
    pub logo: Option<String>,
    pub country: Option<String>,
}

/// Sector summary returned by the API.
#[derive(Debug, Serialize)]
pub struct SectorSummary {
    pub key: String,
    pub label: String,
    pub symbols: Vec<String>,
    pub total_weight: f64,
    pub avg_change_pct: f64,
}

/// Config info returned by /api/config.
#[derive(Debug, Serialize)]
pub struct ConfigInfo {
    pub base_value: f64,
    pub market_cap_weight_pct: u32,
    pub index_stock_count: usize,
    pub benchmark_symbols: Vec<String>,
}
