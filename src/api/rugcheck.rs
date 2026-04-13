use anyhow::{Context, Result};
use serde::Deserialize;

pub struct RugCheckClient {
    client: reqwest::Client,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RugCheckReport {
    pub score: Option<f64>,
    #[serde(default)]
    pub risks: Vec<RiskItem>,
    pub mint_authority: Option<String>,
    pub freeze_authority: Option<String>,
    #[serde(default)]
    pub top_holders: Vec<HolderInfo>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct RiskItem {
    pub name: Option<String>,
    pub level: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct HolderInfo {
    pub address: Option<String>,
    pub pct: Option<f64>,
}

impl RugCheckReport {
    pub fn risk_level(&self) -> &'static str {
        match self.score {
            Some(s) if s >= 80.0 => "LOW",
            Some(s) if s >= 50.0 => "MEDIUM",
            Some(_) => "HIGH",
            None => "UNKNOWN",
        }
    }

    pub fn mint_revoked(&self) -> bool {
        self.mint_authority.as_deref() == Some("") || self.mint_authority.is_none()
    }

    pub fn freeze_revoked(&self) -> bool {
        self.freeze_authority.as_deref() == Some("") || self.freeze_authority.is_none()
    }

    pub fn top_holder_pct(&self) -> f64 {
        self.top_holders.iter().take(10).filter_map(|h| h.pct).sum()
    }
}

impl RugCheckClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_report(&self, mint: &str) -> Result<Option<RugCheckReport>> {
        let url = format!("https://api.rugcheck.xyz/v1/tokens/{mint}/report/summary");

        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let report: RugCheckReport = resp
            .json()
            .await
            .context("Failed to parse RugCheck response")?;

        Ok(Some(report))
    }
}
