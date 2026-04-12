use crate::data::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct TrackedWallet {
    pub address: String,
    pub label: String,
    pub sol_balance: Option<f64>,
    pub recent_txs: Vec<Transaction>,
    pub loading: bool,
}

impl TrackedWallet {
    pub fn new(address: String, label: String) -> Self {
        Self {
            address,
            label,
            sol_balance: None,
            recent_txs: Vec::new(),
            loading: true,
        }
    }
}

#[derive(Debug)]
pub struct WhaleState {
    pub wallets: Vec<TrackedWallet>,
    pub selected: usize,
    pub input_active: bool,
    pub input_buffer: String,
    /// Which field is being edited: 0 = address, 1 = label
    pub input_field: u8,
    pub pending_address: String,
}

impl WhaleState {
    pub fn new() -> Self {
        Self {
            wallets: Vec::new(),
            selected: 0,
            input_active: false,
            input_buffer: String::new(),
            input_field: 0,
            pending_address: String::new(),
        }
    }

    pub fn add_wallet(&mut self, address: String, label: String) {
        // Don't add duplicates
        if self.wallets.iter().any(|w| w.address == address) {
            return;
        }
        self.wallets.push(TrackedWallet::new(address, label));
    }

    pub fn remove_selected(&mut self) {
        if !self.wallets.is_empty() {
            self.wallets.remove(self.selected);
            if self.selected > 0 && self.selected >= self.wallets.len() {
                self.selected = self.wallets.len() - 1;
            }
        }
    }

    pub fn selected_wallet(&self) -> Option<&TrackedWallet> {
        self.wallets.get(self.selected)
    }
}
