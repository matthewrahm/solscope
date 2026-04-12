use anyhow::{Context, Result};
use serde::Deserialize;

pub struct DexScreenerClient {
    client: reqwest::Client,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DexPair {
    #[serde(default)]
    pub dex_id: String,
    pub price_usd: Option<String>,
    #[serde(default)]
    pub price_change: PriceChange,
    pub volume: Option<Volume>,
    pub liquidity: Option<Liquidity>,
    pub fdv: Option<f64>,
    pub market_cap: Option<f64>,
    pub base_token: Option<BaseToken>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct PriceChange {
    pub h1: Option<f64>,
    pub h6: Option<f64>,
    pub h24: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Volume {
    pub h24: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Liquidity {
    pub usd: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BaseToken {
    pub name: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Deserialize)]
struct DexResponse {
    pairs: Option<Vec<DexPair>>,
}

impl DexScreenerClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_token_info(&self, mint: &str) -> Result<Option<DexPair>> {
        let url = format!("https://api.dexscreener.com/latest/dex/tokens/{mint}");

        let resp: DexResponse = self
            .client
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .context("Failed to parse DexScreener response")?;

        // Return the pair with highest liquidity
        Ok(resp.pairs.and_then(|mut pairs| {
            pairs.sort_by(|a, b| {
                let liq_a = a.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
                let liq_b = b.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
                liq_b.partial_cmp(&liq_a).unwrap()
            });
            pairs.into_iter().next()
        }))
    }
}
