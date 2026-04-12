use anyhow::{Context, Result};
use serde::Deserialize;

use crate::data::token::TokenAsset;
use crate::data::transaction::Transaction;

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

    /// Fetch parsed transaction history using Helius enhanced API
    pub async fn get_parsed_transactions(
        &self,
        wallet: &str,
        limit: usize,
    ) -> Result<Vec<Transaction>> {
        // Step 1: get recent signatures via RPC
        let sig_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignaturesForAddress",
            "params": [wallet, { "limit": limit }]
        });

        let sig_resp: RpcResponse<Vec<SignatureInfo>> = self
            .client
            .post(&self.rpc_url)
            .json(&sig_body)
            .send()
            .await?
            .json()
            .await
            .context("Failed to parse getSignaturesForAddress response")?;

        if let Some(err) = sig_resp.error {
            anyhow::bail!("RPC error {}: {}", err.code, err.message);
        }

        let sigs: Vec<String> = sig_resp
            .result
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.signature)
            .collect();

        if sigs.is_empty() {
            return Ok(Vec::new());
        }

        // Step 2: parse transactions via Helius enhanced API
        let parse_url = format!(
            "https://api.helius.xyz/v0/transactions?api-key={}",
            self.api_url.split("api-key=").last().unwrap_or("")
        );

        let mut all_txs = Vec::new();

        // Helius accepts up to 100 signatures per batch
        for chunk in sigs.chunks(100) {
            let resp: Vec<HeliusParsedTx> = self
                .client
                .post(&parse_url)
                .json(&serde_json::json!({ "transactions": chunk }))
                .send()
                .await?
                .json()
                .await
                .context("Failed to parse Helius enhanced transactions")?;

            for tx in resp {
                all_txs.push(Transaction::from_helius(tx, wallet));
            }
        }

        Ok(all_txs)
    }
}

// -- Signature types --

#[derive(Deserialize)]
struct SignatureInfo {
    signature: String,
}

// -- Helius enhanced transaction types --

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HeliusParsedTx {
    #[serde(default)]
    pub signature: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub fee: u64,
    #[serde(default)]
    pub fee_payer: String,
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub token_transfers: Vec<HeliusTokenTransfer>,
    #[serde(default)]
    pub native_transfers: Vec<HeliusNativeTransfer>,
    #[serde(default)]
    pub events: serde_json::Value,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HeliusTokenTransfer {
    #[serde(default)]
    pub mint: String,
    #[serde(default)]
    pub from_user_account: String,
    #[serde(default)]
    pub to_user_account: String,
    pub token_amount: Option<f64>,
    #[serde(default)]
    pub token_standard: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HeliusNativeTransfer {
    #[serde(default)]
    pub from_user_account: String,
    #[serde(default)]
    pub to_user_account: String,
    pub amount: Option<u64>,
}
