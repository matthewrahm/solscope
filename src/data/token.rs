/// A fungible token held by a wallet
#[allow(dead_code)]
pub struct TokenAsset {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    /// Human-readable balance (decimals already applied)
    pub balance: f64,
    pub decimals: u8,
}
