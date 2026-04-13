use ratatui::{
    layout::Constraint,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::data::portfolio::Holding;
use crate::tui::theme;

pub fn render_token_table<'a>(
    holdings: &'a [Holding],
    sol_balance: f64,
    sol_price: f64,
    selected: usize,
) -> (Table<'a>, TableState) {
    let mut rows = Vec::new();

    // SOL row first
    let sol_value = sol_balance * sol_price;
    let sol_row = Row::new(vec![
        Cell::from(Line::from(vec![Span::styled(
            "SOL",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )])),
        Cell::from(format_balance(sol_balance)).style(Style::default().fg(theme::TEXT_PRIMARY)),
        Cell::from(format_price(sol_price)).style(Style::default().fg(theme::TEXT_SECONDARY)),
        Cell::from(format_value(sol_value)).style(Style::default().fg(theme::TEXT_PRIMARY)),
    ]);
    rows.push(sol_row);

    // Token rows
    for h in holdings {
        if h.value_usd < 0.01 && h.price_usd == 0.0 {
            continue;
        }

        let symbol_style = Style::default().fg(theme::TEXT_PRIMARY);

        let row = Row::new(vec![
            Cell::from(truncate(&h.symbol, 10)).style(symbol_style),
            Cell::from(format_balance(h.balance)).style(Style::default().fg(theme::TEXT_PRIMARY)),
            Cell::from(format_price(h.price_usd)).style(Style::default().fg(theme::TEXT_SECONDARY)),
            Cell::from(format_value(h.value_usd)).style(Style::default().fg(theme::TEXT_PRIMARY)),
        ]);
        rows.push(row);
    }

    let header = Row::new(vec![
        Cell::from("TOKEN").style(Style::default().fg(theme::TEXT_MUTED)),
        Cell::from("BALANCE").style(Style::default().fg(theme::TEXT_MUTED)),
        Cell::from("PRICE").style(Style::default().fg(theme::TEXT_MUTED)),
        Cell::from("VALUE").style(Style::default().fg(theme::TEXT_MUTED)),
    ])
    .height(1)
    .bottom_margin(1);

    let widths = [
        Constraint::Min(12),
        Constraint::Min(14),
        Constraint::Min(14),
        Constraint::Min(14),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" Holdings ")
                .title_style(Style::default().fg(theme::ACCENT)),
        )
        .row_highlight_style(
            Style::default()
                .bg(theme::BG_ELEVATED)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    let mut state = TableState::default();
    state.select(Some(selected));

    (table, state)
}

fn format_balance(amount: f64) -> String {
    if amount >= 1_000_000.0 {
        format!("{:.1}M", amount / 1_000_000.0)
    } else if amount >= 1_000.0 {
        format!("{:.1}K", amount / 1_000.0)
    } else if amount >= 1.0 {
        format!("{:.4}", amount)
    } else {
        format!("{:.6}", amount)
    }
}

fn format_price(price: f64) -> String {
    if price == 0.0 {
        "-".to_string()
    } else if price >= 1.0 {
        format!("${:.2}", price)
    } else if price >= 0.01 {
        format!("${:.4}", price)
    } else {
        format!("${:.8}", price)
    }
}

fn format_value(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("${:.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.2}K", value / 1_000.0)
    } else if value >= 0.01 {
        format!("${:.2}", value)
    } else if value > 0.0 {
        format!("${:.6}", value)
    } else {
        "$0.00".to_string()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
