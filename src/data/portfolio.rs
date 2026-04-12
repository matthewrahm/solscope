use std::collections::HashMap;
use std::fmt;

use crate::app::SortMode;
use crate::data::token::TokenAsset;

#[allow(dead_code)]
pub struct Holding {
    pub symbol: String,
    pub name: String,
    pub balance: f64,
    pub price_usd: f64,
    pub value_usd: f64,
}

pub struct Portfolio {
    pub sol_balance: f64,
    pub sol_price: f64,
    pub sol_value: f64,
    pub holdings: Vec<Holding>,
    pub total_value: f64,
}

impl Portfolio {
    pub fn build(
        sol_balance: f64,
        sol_price: f64,
        assets: Vec<TokenAsset>,
        prices: &HashMap<String, f64>,
    ) -> Self {
        let sol_value = sol_balance * sol_price;

        let mut holdings: Vec<Holding> = assets
            .into_iter()
            .map(|asset| {
                let price_usd = prices.get(&asset.mint).copied().unwrap_or(0.0);
                let value_usd = asset.balance * price_usd;
                Holding {
                    symbol: asset.symbol,
                    name: asset.name,
                    balance: asset.balance,
                    price_usd,
                    value_usd,
                }
            })
            .collect();

        // Sort by USD value descending
        holdings.sort_by(|a, b| b.value_usd.partial_cmp(&a.value_usd).unwrap());

        let token_value: f64 = holdings.iter().map(|h| h.value_usd).sum();
        let total_value = sol_value + token_value;

        Self {
            sol_balance,
            sol_price,
            sol_value,
            holdings,
            total_value,
        }
    }

    pub fn sort_by(&mut self, mode: &SortMode) {
        match mode {
            SortMode::Value => {
                self.holdings
                    .sort_by(|a, b| b.value_usd.partial_cmp(&a.value_usd).unwrap());
            }
            SortMode::Name => {
                self.holdings
                    .sort_by(|a, b| a.symbol.to_lowercase().cmp(&b.symbol.to_lowercase()));
            }
            SortMode::Balance => {
                self.holdings
                    .sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap());
            }
        }
    }
}

impl fmt::Display for Portfolio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  PORTFOLIO")?;
        writeln!(f, "  {}", "-".repeat(60))?;
        writeln!(
            f,
            "  SOL          {:>12.4}    ${:<10.2}  ${:.2}",
            self.sol_balance, self.sol_price, self.sol_value
        )?;
        writeln!(f, "  {}", "-".repeat(60))?;
        writeln!(
            f,
            "  {:<12} {:>12}    {:<12}  {}",
            "TOKEN", "AMOUNT", "PRICE", "VALUE"
        )?;
        writeln!(f, "  {}", "-".repeat(60))?;

        for h in &self.holdings {
            if h.value_usd < 0.01 && h.price_usd == 0.0 {
                continue; // skip dust with no price data
            }
            writeln!(
                f,
                "  {:<12} {:>12.4}    ${:<10.6}  ${:.2}",
                truncate(&h.symbol, 12),
                h.balance,
                h.price_usd,
                h.value_usd
            )?;
        }

        writeln!(f, "  {}", "-".repeat(60))?;
        writeln!(f, "  TOTAL {:>52}", format!("${:.2}", self.total_value))?;

        Ok(())
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
