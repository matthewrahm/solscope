use anyhow::{Context, Result};
use serde::Deserialize;

use crate::data::token::TokenAsset;

pub struct HeliusClient {
    client: reqwest::Client,
    rpc_url: String,
    api_url: String,
}

// -- RPC response types --

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

// -- DAS (Digital Asset Standard) response types --

#[derive(Deserialize)]
struct DasResponse {
    result: Option<DasResult>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct DasResult {
    items: Vec<DasItem>,
    #[serde(default)]
    total: u32,
    #[serde(default)]
    limit: u32,
}

#[derive(Deserialize)]
struct DasItem {
    id: String,
    content: Option<DasContent>,
    token_info: Option<DasTokenInfo>,
}

#[derive(Deserialize)]
struct DasContent {
    metadata: Option<DasMetadata>,
}

#[derive(Deserialize)]
struct DasMetadata {
    name: Option<String>,
    symbol: Option<String>,
}

#[derive(Deserialize)]
struct DasTokenInfo {
    balance: Option<u64>,
    decimals: Option<u8>,
    associated_token_address: Option<String>,
}

impl HeliusClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            rpc_url: format!("https://mainnet.helius-rpc.com/?api-key={api_key}"),
            api_url: format!("https://api.helius.xyz/v0?api-key={api_key}"),
        }
    }

    /// Fetch native SOL balance in lamports, return as SOL (f64)
    pub async fn get_sol_balance(&self, wallet: &str) -> Result<f64> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [wallet]
        });

        let resp: RpcResponse<serde_json::Value> = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await
            .context("Failed to parse getBalance response")?;

        if let Some(err) = resp.error {
            anyhow::bail!("RPC error {}: {}", err.code, err.message);
        }

        let lamports = resp
            .result
            .and_then(|r| r.get("value").and_then(|v| v.as_u64()))
            .unwrap_or(0);

        Ok(lamports as f64 / 1_000_000_000.0)
    }

    /// Fetch all fungible token holdings using Helius DAS API
    pub async fn get_assets_by_owner(&self, wallet: &str) -> Result<Vec<TokenAsset>> {
        let mut all_items = Vec::new();
        let mut page = 1;

        loop {
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getAssetsByOwner",
                "params": {
                    "ownerAddress": wallet,
                    "page": page,
                    "limit": 1000,
                    "displayOptions": {
                        "showFungible": true,
                        "showNativeBalance": false
                    }
                }
            });

            let resp: DasResponse = self
                .client
                .post(&self.rpc_url)
                .json(&body)
                .send()
                .await?
                .json()
                .await
                .context("Failed to parse getAssetsByOwner response")?;

            if let Some(err) = resp.error {
                anyhow::bail!("RPC error {}: {}", err.code, err.message);
            }

            let result = resp.result.context("Missing result in DAS response")?;
            let count = result.items.len();
            all_items.extend(result.items);

            if count < 1000 {
                break;
            }
            page += 1;
        }

        let assets = all_items
            .into_iter()
            .filter_map(|item| {
                let token_info = item.token_info?;
                let balance = token_info.balance.unwrap_or(0);
                if balance == 0 {
                    return None;
                }
                let decimals = token_info.decimals.unwrap_or(0);
                let metadata = item.content.and_then(|c| c.metadata);

                Some(TokenAsset {
                    mint: item.id,
                    symbol: metadata
                        .as_ref()
                        .and_then(|m| m.symbol.clone())
                        .unwrap_or_else(|| "???".to_string()),
                    name: metadata
                        .as_ref()
                        .and_then(|m| m.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    balance: balance as f64 / 10_f64.powi(decimals as i32),
                    decimals,
                })
            })
            .collect();

        Ok(assets)
    }
}
