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
    let events = EventHandler::new(100); // 100ms tick rate

    // Channel for background data fetches
    let (tx, mut rx) = mpsc::channel::<DataMsg>(16);

    // Initial data fetch
    spawn_fetch(tx.clone(), api_key.clone(), wallet.clone());

    loop {
        // Draw
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Check for data updates (non-blocking)
        if let Ok(msg) = rx.try_recv() {
            match msg {
                DataMsg::Portfolio(p) => app.set_portfolio(p),
                DataMsg::Error(e) => {
                    // For now just mark as not loading — could show error in UI later
                    app.loading = false;
                    eprintln!("Data fetch error: {e}");
                }
            }
        }

        // Handle events
        match events.next()? {
            AppEvent::Key(key) => {
                // Only handle key press events (not release/repeat)
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
            spawn_fetch(tx.clone(), api_key.clone(), wallet.clone());
        }
    }

    Ok(())
}

fn spawn_fetch(tx: mpsc::Sender<DataMsg>, api_key: String, wallet: String) {
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
