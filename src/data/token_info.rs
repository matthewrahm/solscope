use crate::api::dexscreener::DexPair;
use crate::api::rugcheck::RugCheckReport;

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub price_usd: f64,
    pub price_change_1h: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub market_cap: Option<f64>,
    pub fdv: Option<f64>,
    pub volume_24h: Option<f64>,
    pub liquidity: Option<f64>,
    pub dex: String,
    pub security: Option<SecurityInfo>,
}

#[derive(Debug, Clone)]
pub struct SecurityInfo {
    pub risk_level: String,
    pub score: Option<f64>,
    pub mint_revoked: bool,
    pub freeze_revoked: bool,
    pub top_10_pct: f64,
    pub risks: Vec<String>,
}

impl TokenInfo {
    pub fn from_dex_and_rug(
        mint: &str,
        pair: Option<DexPair>,
        rug: Option<RugCheckReport>,
    ) -> Option<Self> {
        let pair = pair?;

        let price_usd = pair
            .price_usd
            .as_ref()
            .and_then(|p| p.parse::<f64>().ok())
            .unwrap_or(0.0);

        let base = pair.base_token.as_ref();
        let name = base
            .and_then(|b| b.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let symbol = base
            .and_then(|b| b.symbol.clone())
            .unwrap_or_else(|| "???".to_string());

        let security = rug.map(|r| SecurityInfo {
            risk_level: r.risk_level().to_string(),
            score: r.score,
            mint_revoked: r.mint_revoked(),
            freeze_revoked: r.freeze_revoked(),
            top_10_pct: r.top_holder_pct(),
            risks: r
                .risks
                .iter()
                .filter_map(|risk| {
                    let name = risk.name.as_deref().unwrap_or("");
                    let level = risk.level.as_deref().unwrap_or("");
                    if level == "error" || level == "warn" {
                        Some(format!("[{level}] {name}"))
                    } else {
                        None
                    }
                })
                .collect(),
        });

        Some(TokenInfo {
            mint: mint.to_string(),
            name,
            symbol,
            price_usd,
            price_change_1h: pair.price_change.h1,
            price_change_24h: pair.price_change.h24,
            market_cap: pair.market_cap,
            fdv: pair.fdv,
            volume_24h: pair.volume.and_then(|v| v.h24),
            liquidity: pair.liquidity.and_then(|l| l.usd),
            dex: pair.dex_id,
            security,
        })
    }
}
