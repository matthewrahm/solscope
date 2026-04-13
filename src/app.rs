use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::data::cache::PriceHistory;
use crate::data::portfolio::Portfolio;
use crate::data::token_info::TokenInfo;
use crate::data::transaction::Transaction;
use crate::data::whale::WhaleState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Portfolio,
    Transactions,
    Whales,
    TokenLookup,
    Help,
}

impl Screen {
    pub fn label(&self) -> &'static str {
        match self {
            Screen::Portfolio => "Portfolio",
            Screen::Transactions => "Transactions",
            Screen::Whales => "Whales",
            Screen::TokenLookup => "Token Lookup",
            Screen::Help => "Help",
        }
    }
}

pub const TABS: [Screen; 4] = [
    Screen::Portfolio,
    Screen::Transactions,
    Screen::Whales,
    Screen::TokenLookup,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Value,
    Name,
    Balance,
}

pub struct App {
    pub screen: Screen,
    pub prev_screen: Screen,
    pub wallet: String,

    // Portfolio
    pub portfolio: Option<Portfolio>,
    pub table_selected: usize,
    pub table_len: usize,
    pub sort_mode: SortMode,

    // Transactions
    pub transactions: Option<Vec<Transaction>>,
    pub tx_selected: usize,

    // Token lookup
    pub token_search_input: String,
    pub token_input_active: bool,
    pub token_info: Option<TokenInfo>,
    pub token_loading: bool,
    pub token_lookup_trigger: Option<String>,

    // Whale tracker
    pub whale_state: WhaleState,
    pub whale_fetch_trigger: Option<String>,

    // Price tracking
    pub price_history: PriceHistory,

    // General
    pub should_quit: bool,
    pub should_refresh: bool,
    pub loading: bool,
    pub last_refresh: Option<chrono::DateTime<chrono::Utc>>,
}

impl App {
    pub fn new(wallet: String) -> Self {
        Self {
            screen: Screen::Portfolio,
            prev_screen: Screen::Portfolio,
            wallet,
            portfolio: None,
            table_selected: 0,
            table_len: 0,
            sort_mode: SortMode::Value,
            transactions: None,
            tx_selected: 0,
            token_search_input: String::new(),
            token_input_active: false,
            token_info: None,
            token_loading: false,
            token_lookup_trigger: None,
            whale_state: WhaleState::new(),
            whale_fetch_trigger: None,
            price_history: PriceHistory::new(60), // 60 samples for sparkline
            should_quit: false,
            should_refresh: false,
            loading: true,
            last_refresh: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        // Input modes capture all keys
        if self.token_input_active {
            self.handle_token_input(key);
            return;
        }
        if self.whale_state.input_active {
            self.handle_whale_input(key);
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => {
                if self.screen == Screen::Help {
                    self.screen = self.prev_screen;
                } else {
                    self.prev_screen = self.screen;
                    self.screen = Screen::Help;
                }
            }
            KeyCode::Esc => {
                if self.screen == Screen::Help {
                    self.screen = self.prev_screen;
                }
            }

            // Tab switching
            KeyCode::Char('1') => self.screen = Screen::Portfolio,
            KeyCode::Char('2') => self.screen = Screen::Transactions,
            KeyCode::Char('3') => self.screen = Screen::Whales,
            KeyCode::Char('4') => self.screen = Screen::TokenLookup,

            // Navigation
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('g') => {
                self.table_selected = 0;
                self.tx_selected = 0;
                self.whale_state.selected = 0;
            }
            KeyCode::Char('G') => self.jump_to_bottom(),

            // Actions
            KeyCode::Char('r') => self.should_refresh = true,
            KeyCode::Char('s') => self.cycle_sort(),

            // Token lookup search
            KeyCode::Char('/') if self.screen == Screen::TokenLookup => {
                self.token_input_active = true;
                self.token_search_input.clear();
            }

            // Whale tracker actions
            KeyCode::Char('a') if self.screen == Screen::Whales => {
                self.whale_state.input_active = true;
                self.whale_state.input_field = 0;
                self.whale_state.input_buffer.clear();
                self.whale_state.pending_address.clear();
            }
            KeyCode::Char('d') if self.screen == Screen::Whales => {
                self.whale_state.remove_selected();
            }

            _ => {}
        }
    }

    fn handle_token_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.token_input_active = false;
            }
            KeyCode::Enter => {
                self.token_input_active = false;
                if !self.token_search_input.is_empty() {
                    self.token_loading = true;
                    self.token_info = None;
                    self.token_lookup_trigger = Some(self.token_search_input.clone());
                }
            }
            KeyCode::Backspace => {
                self.token_search_input.pop();
            }
            KeyCode::Char(c) => {
                self.token_search_input.push(c);
            }
            _ => {}
        }
    }

    fn handle_whale_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.whale_state.input_active = false;
                self.whale_state.input_buffer.clear();
                self.whale_state.pending_address.clear();
            }
            KeyCode::Enter => {
                if self.whale_state.input_field == 0 {
                    // Address entered, now ask for label
                    if !self.whale_state.input_buffer.is_empty() {
                        self.whale_state.pending_address = self.whale_state.input_buffer.clone();
                        self.whale_state.input_buffer.clear();
                        self.whale_state.input_field = 1;
                    }
                } else {
                    // Label entered, add the wallet
                    let address = self.whale_state.pending_address.clone();
                    let label = if self.whale_state.input_buffer.is_empty() {
                        format!("Wallet {}", self.whale_state.wallets.len() + 1)
                    } else {
                        self.whale_state.input_buffer.clone()
                    };
                    self.whale_state.add_wallet(address.clone(), label);
                    self.whale_state.input_active = false;
                    self.whale_state.input_buffer.clear();
                    self.whale_state.pending_address.clear();
                    // Trigger fetch for the new wallet
                    self.whale_fetch_trigger = Some(address);
                }
            }
            KeyCode::Backspace => {
                self.whale_state.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.whale_state.input_buffer.push(c);
            }
            _ => {}
        }
    }

    fn select_next(&mut self) {
        match self.screen {
            Screen::Portfolio => {
                if self.table_len > 0 && self.table_selected < self.table_len - 1 {
                    self.table_selected += 1;
                }
            }
            Screen::Transactions => {
                let len = self.transactions.as_ref().map(|t| t.len()).unwrap_or(0);
                if len > 0 && self.tx_selected < len - 1 {
                    self.tx_selected += 1;
                }
            }
            Screen::Whales => {
                let len = self.whale_state.wallets.len();
                if len > 0 && self.whale_state.selected < len - 1 {
                    self.whale_state.selected += 1;
                }
            }
            _ => {}
        }
    }

    fn select_prev(&mut self) {
        match self.screen {
            Screen::Portfolio => {
                if self.table_selected > 0 {
                    self.table_selected -= 1;
                }
            }
            Screen::Transactions => {
                if self.tx_selected > 0 {
                    self.tx_selected -= 1;
                }
            }
            Screen::Whales => {
                if self.whale_state.selected > 0 {
                    self.whale_state.selected -= 1;
                }
            }
            _ => {}
        }
    }

    fn jump_to_bottom(&mut self) {
        match self.screen {
            Screen::Portfolio => {
                if self.table_len > 0 {
                    self.table_selected = self.table_len - 1;
                }
            }
            Screen::Transactions => {
                let len = self.transactions.as_ref().map(|t| t.len()).unwrap_or(0);
                if len > 0 {
                    self.tx_selected = len - 1;
                }
            }
            Screen::Whales => {
                let len = self.whale_state.wallets.len();
                if len > 0 {
                    self.whale_state.selected = len - 1;
                }
            }
            _ => {}
        }
    }

    fn cycle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Value => SortMode::Name,
            SortMode::Name => SortMode::Balance,
            SortMode::Balance => SortMode::Value,
        };
        if let Some(ref mut portfolio) = self.portfolio {
            portfolio.sort_by(&self.sort_mode);
        }
    }

    pub fn set_portfolio(&mut self, mut portfolio: Portfolio) {
        portfolio.sort_by(&self.sort_mode);
        self.table_len = portfolio
            .holdings
            .iter()
            .filter(|h| h.value_usd >= 0.01 || h.price_usd > 0.0)
            .count()
            + 1;

        // Record SOL price for sparkline
        self.price_history.record(
            "So11111111111111111111111111111111111111112",
            portfolio.sol_price,
        );

        self.portfolio = Some(portfolio);
        self.loading = false;
        self.last_refresh = Some(chrono::Utc::now());
    }

    pub fn set_transactions(&mut self, txs: Vec<Transaction>) {
        self.transactions = Some(txs);
    }

    pub fn set_token_info(&mut self, info: Option<TokenInfo>) {
        self.token_info = info;
        self.token_loading = false;
    }

    pub fn update_whale_data(&mut self, address: &str, sol_balance: f64, txs: Vec<Transaction>) {
        if let Some(wallet) = self
            .whale_state
            .wallets
            .iter_mut()
            .find(|w| w.address == address)
        {
            wallet.sol_balance = Some(sol_balance);
            wallet.recent_txs = txs;
            wallet.loading = false;
        }
    }

    pub fn last_refresh_label(&self) -> String {
        match self.last_refresh {
            Some(t) => {
                let elapsed = chrono::Utc::now() - t;
                let secs = elapsed.num_seconds();
                if secs < 60 {
                    format!("{secs}s ago")
                } else {
                    format!("{}m ago", secs / 60)
                }
            }
            None => "loading...".to_string(),
        }
    }
}
