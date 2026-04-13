# solscope

A real-time terminal analytics dashboard for Solana wallets, built in Rust.

```
solscope --wallet <SOLANA_ADDRESS>
```

## Features

**Portfolio** -- Token holdings with USD values, sortable by value/name/balance. SOL price sparkline with 30s auto-refresh.

**Transactions** -- Last 50 decoded transactions. Swaps show token pairs and amounts. Transfers show direction and counterparty.

**Whale Tracker** -- Monitor any wallet in real-time. Add/remove addresses, see SOL balance and decoded activity feed.

**Token Lookup** -- Paste any mint address. Market data from DexScreener (price, mcap, volume, liquidity) and security audit from RugCheck (risk score, mint/freeze authority, top holder concentration).

## Architecture

```
                User Input (crossterm)
                       |
                  Event Loop
                   /        \
          App State           Background Tasks (tokio)
          Machine               /    |    \    \
             |            Helius  Jupiter  Dex  RugCheck
             |               \    |    /    /
          ratatui          mpsc channels
          render              |
             |            App::update
          Terminal
```

- **Async pipeline**: API calls run on background tokio tasks, send results through `mpsc` channels. UI never blocks.
- **4 API integrations**: Helius RPC + DAS (portfolio, transactions), Jupiter v3 (prices), DexScreener (market data), RugCheck (security).
- **State machine**: `App` struct with `Screen` enum drives all rendering and input handling.

## Keybindings

| Key | Action |
|-----|--------|
| `1-4` | Switch tabs |
| `j/k` | Navigate up/down |
| `g/G` | Jump to top/bottom |
| `s` | Cycle sort (portfolio) |
| `/` | Search token (token lookup tab) |
| `a` | Add wallet (whale tracker tab) |
| `d` | Remove wallet (whale tracker tab) |
| `r` | Refresh data |
| `?` | Toggle help |
| `q` | Quit |

## Setup

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- A [Helius](https://helius.dev/) API key (free tier works)

### Install and run

```sh
git clone https://github.com/matthewrahm/solscope.git
cd solscope

# Set your API key
cp .env.example .env
# Edit .env and add your HELIUS_API_KEY

# Run
cargo run -- --wallet <SOLANA_ADDRESS>

# Or pass the key directly
cargo run -- --wallet <SOLANA_ADDRESS> --api-key <YOUR_KEY>
```

### Build release binary

```sh
cargo build --release
./target/release/solscope --wallet <SOLANA_ADDRESS>
```

## Project Structure

```
src/
  main.rs              # Entry point, event loop, background task spawning
  app.rs               # App state machine, Screen enum, key handling
  ui.rs                # Top-level layout: tabs, content, status bar
  error.rs             # Custom error types
  api/
    helius.rs          # Helius RPC + DAS API client
    jupiter.rs         # Jupiter v3 price API
    dexscreener.rs     # DexScreener token pairs
    rugcheck.rs        # RugCheck security reports
  data/
    portfolio.rs       # Portfolio aggregation + sorting
    transaction.rs     # Transaction parsing from Helius enhanced API
    token.rs           # Token asset type
    token_info.rs      # Combined market + security data
    whale.rs           # Whale tracker state
    cache.rs           # Price history for sparklines
  tui/
    event.rs           # Crossterm event polling
    theme.rs           # Color palette (dark theme)
    screens/
      portfolio.rs     # Portfolio tab rendering
      transactions.rs  # Transaction history tab
      whales.rs        # Whale tracker tab
      token_lookup.rs  # Token lookup tab
      help.rs          # Keybindings reference
    widgets/
      token_table.rs   # Sortable token holdings table
      status_bar.rs    # Bottom status bar
```

## Tech Stack

| Crate | Purpose |
|-------|---------|
| `ratatui` | Terminal UI framework |
| `crossterm` | Cross-platform terminal input |
| `tokio` | Async runtime + channels |
| `reqwest` | HTTP client (rustls) |
| `clap` | CLI argument parsing |
| `serde` | JSON deserialization |
| `chrono` | Timestamp handling |

## License

MIT
