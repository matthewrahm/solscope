mod api;
mod app;
mod data;
mod error;
mod tui;
mod ui;

use std::io;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::KeyEventKind,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::CrosstermBackend;
use tokio::sync::mpsc;

use crate::app::App;
use crate::data::portfolio::Portfolio;
use crate::data::token_info::TokenInfo;
use crate::data::transaction::Transaction;
use crate::tui::event::{AppEvent, EventHandler};

#[derive(Parser, Debug)]
#[command(name = "solscope", version, about = "Terminal analytics dashboard for Solana wallets")]
struct Args {
    /// Solana wallet address to analyze
    #[arg(short, long)]
    wallet: String,

    /// Helius API key (or set HELIUS_API_KEY env var)
    #[arg(short = 'k', long)]
    api_key: Option<String>,
}

enum DataMsg {
    Portfolio(Portfolio),
    Transactions(Vec<Transaction>),
    TokenInfo(Option<TokenInfo>),
    WhaleData {
        address: String,
        sol_balance: f64,
        txs: Vec<Transaction>,
    },
    #[allow(dead_code)]
    Error(String),
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let args = Args::parse();

    let api_key = args
        .api_key
        .or_else(|| std::env::var("HELIUS_API_KEY").ok())
        .expect("Helius API key required: pass --api-key or set HELIUS_API_KEY");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = run_app(&mut terminal, args.wallet, api_key).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

async fn run_app(
    terminal: &mut ratatui::Terminal<CrosstermBackend<io::Stdout>>,
    wallet: String,
    api_key: String,
) -> Result<()> {
    let mut app = App::new(wallet.clone());
    let events = EventHandler::new(100);
    let mut last_auto_refresh = std::time::Instant::now();
    let auto_refresh_interval = std::time::Duration::from_secs(30);

    let (tx, mut rx) = mpsc::channel::<DataMsg>(32);

    // Initial data fetches — portfolio and transactions in parallel
    spawn_portfolio_fetch(tx.clone(), api_key.clone(), wallet.clone());
    spawn_transaction_fetch(tx.clone(), api_key.clone(), wallet.clone());

    loop {
        // Draw
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Drain all pending data messages
        while let Ok(msg) = rx.try_recv() {
            match msg {
                DataMsg::Portfolio(p) => app.set_portfolio(p),
                DataMsg::Transactions(txs) => app.set_transactions(txs),
                DataMsg::TokenInfo(info) => app.set_token_info(info),
                DataMsg::WhaleData { address, sol_balance, txs } => {
                    app.update_whale_data(&address, sol_balance, txs);
                }
                DataMsg::Error(_) => {
                    app.loading = false;
                }
            }
        }

        // Handle events
        match events.next()? {
            AppEvent::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
            AppEvent::Tick => {}
        }

        if app.should_quit {
            break;
        }

        if app.should_refresh {
            app.should_refresh = false;
            app.loading = true;
            spawn_portfolio_fetch(tx.clone(), api_key.clone(), wallet.clone());
            spawn_transaction_fetch(tx.clone(), api_key.clone(), wallet.clone());
            last_auto_refresh = std::time::Instant::now();
        }

        // Auto-refresh every 30s for sparkline data
        if last_auto_refresh.elapsed() >= auto_refresh_interval {
            spawn_portfolio_fetch(tx.clone(), api_key.clone(), wallet.clone());
            last_auto_refresh = std::time::Instant::now();
        }

        // Token lookup trigger
        if let Some(mint) = app.token_lookup_trigger.take() {
            spawn_token_lookup(tx.clone(), mint);
        }

        // Whale fetch trigger
        if let Some(address) = app.whale_fetch_trigger.take() {
            spawn_whale_fetch(tx.clone(), api_key.clone(), address);
        }
    }

    Ok(())
}

fn spawn_portfolio_fetch(tx: mpsc::Sender<DataMsg>, api_key: String, wallet: String) {
    tokio::spawn(async move {
        match fetch_portfolio(&api_key, &wallet).await {
            Ok(portfolio) => {
                let _ = tx.send(DataMsg::Portfolio(portfolio)).await;
            }
            Err(e) => {
                let _ = tx.send(DataMsg::Error(e.to_string())).await;
            }
        }
    });
}

fn spawn_transaction_fetch(tx: mpsc::Sender<DataMsg>, api_key: String, wallet: String) {
    tokio::spawn(async move {
        match fetch_transactions(&api_key, &wallet).await {
            Ok(txs) => {
                let _ = tx.send(DataMsg::Transactions(txs)).await;
            }
            Err(e) => {
                let _ = tx.send(DataMsg::Error(e.to_string())).await;
            }
        }
    });
}

fn spawn_token_lookup(tx: mpsc::Sender<DataMsg>, mint: String) {
    tokio::spawn(async move {
        match fetch_token_info(&mint).await {
            Ok(info) => {
                let _ = tx.send(DataMsg::TokenInfo(info)).await;
            }
            Err(_) => {
                let _ = tx.send(DataMsg::TokenInfo(None)).await;
            }
        }
    });
}

fn spawn_whale_fetch(tx: mpsc::Sender<DataMsg>, api_key: String, address: String) {
    tokio::spawn(async move {
        let helius = api::helius::HeliusClient::new(&api_key);
        let sol_balance = helius.get_sol_balance(&address).await.unwrap_or(0.0);
        let txs = helius
            .get_parsed_transactions(&address, 20)
            .await
            .unwrap_or_default();
        let _ = tx
            .send(DataMsg::WhaleData {
                address,
                sol_balance,
                txs,
            })
            .await;
    });
}

async fn fetch_portfolio(api_key: &str, wallet: &str) -> Result<Portfolio> {
    let helius = api::helius::HeliusClient::new(api_key);
    let jupiter = api::jupiter::JupiterClient::new();

    let sol_balance = helius.get_sol_balance(wallet).await?;
    let assets = helius.get_assets_by_owner(wallet).await?;

    let sol_price = jupiter
        .get_price("So11111111111111111111111111111111111111112")
        .await?;

    let mints: Vec<&str> = assets.iter().map(|a| a.mint.as_str()).collect();
    let prices = if !mints.is_empty() {
        jupiter.get_prices(&mints).await?
    } else {
        std::collections::HashMap::new()
    };

    Ok(Portfolio::build(sol_balance, sol_price, assets, &prices))
}

async fn fetch_transactions(api_key: &str, wallet: &str) -> Result<Vec<Transaction>> {
    let helius = api::helius::HeliusClient::new(api_key);
    helius.get_parsed_transactions(wallet, 50).await
}

async fn fetch_token_info(mint: &str) -> Result<Option<TokenInfo>> {
    let dex = api::dexscreener::DexScreenerClient::new();
    let rug = api::rugcheck::RugCheckClient::new();

    // Fetch both in parallel
    let (dex_result, rug_result) = tokio::join!(dex.get_token_info(mint), rug.get_report(mint));

    Ok(TokenInfo::from_dex_and_rug(
        mint,
        dex_result.ok().flatten(),
        rug_result.ok().flatten(),
    ))
}
