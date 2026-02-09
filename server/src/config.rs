use serde::Deserialize;
use std::collections::HashMap;

const DEFAULT_TOML: &str = include_str!("../../stocks.toml");

#[derive(Debug, Deserialize)]
pub struct StocksConfig {
    pub settings: Settings,
    pub sectors: HashMap<String, Sector>,
    pub benchmarks: Benchmarks,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub base_value: f64,
    pub market_cap_weight_pct: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sector {
    pub label: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Benchmarks {
    pub symbols: Vec<String>,
}

impl StocksConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let toml_str = if let Ok(path) = std::env::var("AIINDEX_STOCKS_PATH") {
            std::fs::read_to_string(&path)?
        } else {
            DEFAULT_TOML.to_string()
        };
        let config: StocksConfig = toml::from_str(&toml_str)?;
        Ok(config)
    }

    /// All index symbols (excludes benchmarks).
    pub fn index_symbols(&self) -> Vec<String> {
        self.sectors
            .values()
            .flat_map(|s| s.symbols.iter().cloned())
            .collect()
    }

    /// All symbols including benchmarks.
    pub fn all_symbols(&self) -> Vec<String> {
        let mut syms = self.index_symbols();
        syms.extend(self.benchmarks.symbols.iter().cloned());
        syms
    }

    /// The blended market-cap percentage as a fraction (0.0–1.0).
    pub fn mcap_pct(&self) -> f64 {
        self.settings.market_cap_weight_pct as f64 / 100.0
    }
}
