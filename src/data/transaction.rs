use crate::api::helius::HeliusParsedTx;

#[derive(Debug, Clone)]
pub enum TxType {
    Swap,
    Transfer,
    NftSale,
    NftMint,
    Unknown,
}

impl TxType {
    pub fn label(&self) -> &'static str {
        match self {
            TxType::Swap => "SWAP",
            TxType::Transfer => "TRANSFER",
            TxType::NftSale => "NFT SALE",
            TxType::NftMint => "NFT MINT",
            TxType::Unknown => "OTHER",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub signature: String,
    pub tx_type: TxType,
    pub source: String,
    pub description: String,
    pub fee_sol: f64,
    pub timestamp: Option<i64>,
    pub details: TxDetails,
}

#[derive(Debug, Clone)]
pub enum TxDetails {
    Swap {
        token_in_symbol: String,
        token_in_amount: f64,
        token_out_symbol: String,
        token_out_amount: f64,
    },
    Transfer {
        direction: TransferDirection,
        token_symbol: String,
        amount: f64,
        counterparty: String,
    },
    NativeSol {
        direction: TransferDirection,
        amount_sol: f64,
        counterparty: String,
    },
    Other {
        summary: String,
    },
}

#[derive(Debug, Clone)]
pub enum TransferDirection {
    Sent,
    Received,
}

impl Transaction {
    pub fn from_helius(tx: HeliusParsedTx, wallet: &str) -> Self {
        let tx_type = match tx.r#type.as_str() {
            "SWAP" => TxType::Swap,
            "TRANSFER" => TxType::Transfer,
            "NFT_SALE" | "NFT_LISTING" => TxType::NftSale,
            "NFT_MINT" | "COMPRESSED_NFT_MINT" => TxType::NftMint,
            _ => TxType::Unknown,
        };

        let details = match &tx_type {
            TxType::Swap => parse_swap_details(&tx, wallet),
            TxType::Transfer => parse_transfer_details(&tx, wallet),
            _ => {
                if !tx.native_transfers.is_empty() {
                    parse_native_transfer(&tx, wallet)
                } else {
                    TxDetails::Other {
                        summary: if tx.description.is_empty() {
                            tx.r#type.clone()
                        } else {
                            tx.description.clone()
                        },
                    }
                }
            }
        };

        Transaction {
            signature: tx.signature,
            tx_type,
            source: tx.source,
            description: tx.description,
            fee_sol: tx.fee as f64 / 1_000_000_000.0,
            timestamp: tx.timestamp,
            details,
        }
    }

    pub fn time_ago(&self) -> String {
        let Some(ts) = self.timestamp else {
            return "unknown".to_string();
        };
        let now = chrono::Utc::now().timestamp();
        let diff = now - ts;

        if diff < 60 {
            format!("{diff}s ago")
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
        }
    }
}

fn parse_swap_details(tx: &HeliusParsedTx, wallet: &str) -> TxDetails {
    let wallet_lower = wallet.to_lowercase();

    // Find tokens sent and received by the wallet
    let mut sent: Option<(&str, f64)> = None;
    let mut received: Option<(&str, f64)> = None;

    for t in &tx.token_transfers {
        let amount = t.token_amount.unwrap_or(0.0);
        if amount == 0.0 {
            continue;
        }
        if t.from_user_account.to_lowercase() == wallet_lower {
            if sent.is_none() || amount > sent.unwrap().1 {
                sent = Some((&t.mint, amount));
            }
        }
        if t.to_user_account.to_lowercase() == wallet_lower {
            if received.is_none() || amount > received.unwrap().1 {
                received = Some((&t.mint, amount));
            }
        }
    }

    // Check native SOL transfers too
    for n in &tx.native_transfers {
        let amount_sol = n.amount.unwrap_or(0) as f64 / 1_000_000_000.0;
        if amount_sol < 0.001 {
            continue;
        }
        if n.from_user_account.to_lowercase() == wallet_lower && sent.is_none() {
            sent = Some(("SOL", amount_sol));
        }
        if n.to_user_account.to_lowercase() == wallet_lower && received.is_none() {
            received = Some(("SOL", amount_sol));
        }
    }

    match (sent, received) {
        (Some((in_mint, in_amt)), Some((out_mint, out_amt))) => TxDetails::Swap {
            token_in_symbol: short_mint(in_mint),
            token_in_amount: in_amt,
            token_out_symbol: short_mint(out_mint),
            token_out_amount: out_amt,
        },
        _ => TxDetails::Other {
            summary: if tx.description.is_empty() {
                "Swap".to_string()
            } else {
                tx.description.clone()
            },
        },
    }
}

fn parse_transfer_details(tx: &HeliusParsedTx, wallet: &str) -> TxDetails {
    let wallet_lower = wallet.to_lowercase();

    // Check token transfers first
    if let Some(t) = tx.token_transfers.first() {
        let amount = t.token_amount.unwrap_or(0.0);
        let is_sender = t.from_user_account.to_lowercase() == wallet_lower;
        let counterparty = if is_sender {
            &t.to_user_account
        } else {
            &t.from_user_account
        };

        return TxDetails::Transfer {
            direction: if is_sender {
                TransferDirection::Sent
            } else {
                TransferDirection::Received
            },
            token_symbol: short_mint(&t.mint),
            amount,
            counterparty: short_address(counterparty),
        };
    }

    // Fallback to native transfers
    parse_native_transfer(tx, wallet)
}

fn parse_native_transfer(tx: &HeliusParsedTx, wallet: &str) -> TxDetails {
    let wallet_lower = wallet.to_lowercase();

    for n in &tx.native_transfers {
        let amount_sol = n.amount.unwrap_or(0) as f64 / 1_000_000_000.0;
        if amount_sol < 0.0001 {
            continue;
        }
        let is_sender = n.from_user_account.to_lowercase() == wallet_lower;
        let counterparty = if is_sender {
            &n.to_user_account
        } else {
            &n.from_user_account
        };

        return TxDetails::NativeSol {
            direction: if is_sender {
                TransferDirection::Sent
            } else {
                TransferDirection::Received
            },
            amount_sol,
            counterparty: short_address(counterparty),
        };
    }

    TxDetails::Other {
        summary: if tx.description.is_empty() {
            "Transfer".to_string()
        } else {
            tx.description.clone()
        },
    }
}

fn short_mint(mint: &str) -> String {
    if mint == "SOL" || mint.len() < 8 {
        return mint.to_string();
    }
    // Return first 4 + last 4 for unknown mints
    format!("{}...{}", &mint[..4], &mint[mint.len() - 4..])
}

fn short_address(addr: &str) -> String {
    if addr.len() < 8 {
        return addr.to_string();
    }
    format!("{}...{}", &addr[..4], &addr[addr.len() - 4..])
}
