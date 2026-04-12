use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::data::portfolio::Portfolio;
use crate::data::token_info::TokenInfo;
use crate::data::transaction::Transaction;

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

        // Token lookup input mode captures all keys
        if self.token_input_active {
            self.handle_token_input(key);
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
            + 1; // +1 for SOL row
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
