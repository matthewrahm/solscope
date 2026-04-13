use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::app::App;
use crate::data::transaction::{TransferDirection, TxDetails, TxType};
use crate::tui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let txs = match &app.transactions {
        Some(txs) if !txs.is_empty() => txs,
        Some(_) => {
            let empty = ratatui::widgets::Paragraph::new("  No transactions found")
                .style(Style::default().fg(theme::TEXT_MUTED));
            frame.render_widget(empty, area);
            return;
        }
        None => {
            let loading = ratatui::widgets::Paragraph::new("  Loading transactions...")
                .style(Style::default().fg(theme::TEXT_MUTED));
            frame.render_widget(loading, area);
            return;
        }
    };

    let sol_price = app.portfolio.as_ref().map(|p| p.sol_price).unwrap_or(0.0);

    let mut rows = Vec::new();

    for tx in txs {
        let type_style = match tx.tx_type {
            TxType::Swap => Style::default().fg(theme::ACCENT),
            TxType::Transfer => Style::default().fg(theme::GREEN),
            TxType::NftSale | TxType::NftMint => Style::default().fg(theme::YELLOW),
            TxType::Unknown => Style::default().fg(theme::TEXT_MUTED),
        };

        let (details_str, value_str) = format_details(&tx.details, sol_price);

        let value_style = if value_str.starts_with('-') {
            Style::default().fg(theme::RED)
        } else if value_str.starts_with('+') {
            Style::default().fg(theme::GREEN)
        } else {
            Style::default().fg(theme::TEXT_SECONDARY)
        };

        let row = Row::new(vec![
            Cell::from(tx.time_ago()).style(Style::default().fg(theme::TEXT_MUTED)),
            Cell::from(tx.tx_type.label()).style(type_style),
            Cell::from(if tx.source.is_empty() {
                "-".to_string()
            } else {
                tx.source.clone()
            })
            .style(Style::default().fg(theme::TEXT_SECONDARY)),
            Cell::from(details_str).style(Style::default().fg(theme::TEXT_PRIMARY)),
            Cell::from(value_str).style(value_style),
        ]);
        rows.push(row);
    }

    let header = Row::new(vec![
        Cell::from("TIME").style(Style::default().fg(theme::TEXT_MUTED)),
        Cell::from("TYPE").style(Style::default().fg(theme::TEXT_MUTED)),
        Cell::from("SOURCE").style(Style::default().fg(theme::TEXT_MUTED)),
        Cell::from("DETAILS").style(Style::default().fg(theme::TEXT_MUTED)),
        Cell::from("VALUE").style(Style::default().fg(theme::TEXT_MUTED)),
    ])
    .height(1)
    .bottom_margin(1);

    let widths = [
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Min(30),
        Constraint::Length(16),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(Line::from(vec![
                    Span::styled(" Transactions ", Style::default().fg(theme::ACCENT)),
                    Span::styled(
                        format!("({}) ", txs.len()),
                        Style::default().fg(theme::TEXT_MUTED),
                    ),
                ])),
        )
        .row_highlight_style(
            Style::default()
                .bg(theme::BG_ELEVATED)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    let mut state = TableState::default();
    state.select(Some(app.tx_selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn format_details(details: &TxDetails, sol_price: f64) -> (String, String) {
    match details {
        TxDetails::Swap {
            token_in_symbol,
            token_in_amount,
            token_out_symbol,
            token_out_amount,
        } => {
            let detail = format!(
                "{} {} -> {} {}",
                format_amount(*token_in_amount),
                token_in_symbol,
                format_amount(*token_out_amount),
                token_out_symbol
            );
            // Estimate value from SOL side of the swap
            let value = if token_in_symbol == "SOL" {
                format_usd(*token_in_amount * sol_price)
            } else if token_out_symbol == "SOL" {
                format_usd(*token_out_amount * sol_price)
            } else {
                String::new()
            };
            (detail, value)
        }
        TxDetails::Transfer {
            direction,
            token_symbol,
            amount,
            counterparty,
        } => {
            let arrow = match direction {
                TransferDirection::Sent => "->",
                TransferDirection::Received => "<-",
            };
            let detail = format!(
                "{} {} {} {}",
                format_amount(*amount),
                token_symbol,
                arrow,
                counterparty
            );
            let value = if token_symbol.contains("SOL") {
                let usd = *amount * sol_price;
                match direction {
                    TransferDirection::Sent => format!("-{}", format_usd(usd)),
                    TransferDirection::Received => format!("+{}", format_usd(usd)),
                }
            } else {
                String::new()
            };
            (detail, value)
        }
        TxDetails::NativeSol {
            direction,
            amount_sol,
            counterparty,
        } => {
            let arrow = match direction {
                TransferDirection::Sent => "->",
                TransferDirection::Received => "<-",
            };
            let detail = format!("{:.4} SOL {} {}", amount_sol, arrow, counterparty);
            let usd = *amount_sol * sol_price;
            let value = match direction {
                TransferDirection::Sent => format!("-{}", format_usd(usd)),
                TransferDirection::Received => format!("+{}", format_usd(usd)),
            };
            (detail, value)
        }
        TxDetails::Other { summary } => {
            let truncated = if summary.len() > 50 {
                format!("{}...", &summary[..47])
            } else {
                summary.clone()
            };
            (truncated, String::new())
        }
    }
}

fn format_amount(amount: f64) -> String {
    if amount >= 1_000_000.0 {
        format!("{:.1}M", amount / 1_000_000.0)
    } else if amount >= 1_000.0 {
        format!("{:.1}K", amount / 1_000.0)
    } else if amount >= 1.0 {
        format!("{:.2}", amount)
    } else {
        format!("{:.4}", amount)
    }
}

fn format_usd(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("${:.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.0}", value)
    } else if value >= 0.01 {
        format!("${:.2}", value)
    } else {
        "$0.00".to_string()
    }
}
