use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::data::portfolio::Portfolio;

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

    pub fn index(&self) -> usize {
        match self {
            Screen::Portfolio => 0,
            Screen::Transactions => 1,
            Screen::Whales => 2,
            Screen::TokenLookup => 3,
            Screen::Help => 4,
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
    pub portfolio: Option<Portfolio>,
    pub table_selected: usize,
    pub table_len: usize,
    pub sort_mode: SortMode,
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
            KeyCode::Char('g') => self.table_selected = 0,
            KeyCode::Char('G') => {
                if self.table_len > 0 {
                    self.table_selected = self.table_len - 1;
                }
            }

            // Actions
            KeyCode::Char('r') => self.should_refresh = true,
            KeyCode::Char('s') => self.cycle_sort(),

            _ => {}
        }
    }

    fn select_next(&mut self) {
        if self.table_len > 0 && self.table_selected < self.table_len - 1 {
            self.table_selected += 1;
        }
    }

    fn select_prev(&mut self) {
        if self.table_selected > 0 {
            self.table_selected -= 1;
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
        // +1 for SOL row
        self.table_len = portfolio
            .holdings
            .iter()
            .filter(|h| h.value_usd >= 0.01 || h.price_usd > 0.0)
            .count()
            + 1;
        self.portfolio = Some(portfolio);
        self.loading = false;
        self.last_refresh = Some(chrono::Utc::now());
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
