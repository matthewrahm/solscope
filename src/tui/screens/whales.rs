use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::App;
use crate::data::transaction::{TransferDirection, TxDetails, TxType};
use crate::tui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let whale = &app.whale_state;

    // Input mode: show add wallet form
    if whale.input_active {
        render_add_form(frame, area, app);
        return;
    }

    if whale.wallets.is_empty() {
        render_empty(frame, area);
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Length(5 + whale.wallets.len() as u16), // wallet list
        Constraint::Min(8),                                  // activity feed
    ])
    .split(area);

    render_wallet_list(frame, chunks[0], app);
    render_activity_feed(frame, chunks[1], app);
}

fn render_empty(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  No wallets tracked yet",
                Style::default().fg(theme::TEXT_SECONDARY),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("a", Style::default().fg(theme::ACCENT)),
            Span::styled(" to add a wallet to track", Style::default().fg(theme::TEXT_MUTED)),
        ]),
    ];

    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(" Whale Tracker ")
            .title_style(Style::default().fg(theme::ACCENT)),
    );
    frame.render_widget(p, area);
}

fn render_add_form(frame: &mut Frame, area: Rect, app: &App) {
    let whale = &app.whale_state;

    let field_label = if whale.input_field == 0 {
        "Wallet Address"
    } else {
        "Label"
    };

    let cursor = format!("{}|", whale.input_buffer);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Add Wallet to Track",
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {field_label}: "), Style::default().fg(theme::ACCENT)),
            Span::styled(&cursor, Style::default().fg(theme::TEXT_PRIMARY)),
        ]),
        Line::from(""),
        if whale.input_field == 0 {
            Line::from(vec![
                Span::styled(
                    "  Paste a Solana wallet address and press Enter",
                    Style::default().fg(theme::TEXT_MUTED),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled("  Address: ", Style::default().fg(theme::TEXT_MUTED)),
                Span::styled(
                    short_addr(&whale.pending_address),
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
            ])
        },
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(theme::ACCENT)),
            Span::styled(": confirm  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled("Esc", Style::default().fg(theme::ACCENT)),
            Span::styled(": cancel", Style::default().fg(theme::TEXT_MUTED)),
        ]),
    ];

    let form = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ACCENT))
            .title(" Add Wallet ")
            .title_style(Style::default().fg(theme::ACCENT)),
    );
    frame.render_widget(form, area);
}

fn render_wallet_list(frame: &mut Frame, area: Rect, app: &App) {
    let whale = &app.whale_state;

    let rows: Vec<Row> = whale
        .wallets
        .iter()
        .map(|w| {
            let balance_str = match w.sol_balance {
                Some(b) => format!("{:.2} SOL", b),
                None if w.loading => "loading...".to_string(),
                None => "-".to_string(),
            };

            let last_activity = w
                .recent_txs
                .first()
                .map(|tx| tx.time_ago())
                .unwrap_or_else(|| {
                    if w.loading {
                        "...".to_string()
                    } else {
                        "-".to_string()
                    }
                });

            Row::new(vec![
                Cell::from(w.label.clone()).style(Style::default().fg(theme::TEXT_PRIMARY)),
                Cell::from(short_addr(&w.address))
                    .style(Style::default().fg(theme::TEXT_SECONDARY)),
                Cell::from(balance_str).style(Style::default().fg(theme::ACCENT)),
                Cell::from(last_activity).style(Style::default().fg(theme::TEXT_MUTED)),
            ])
        })
        .collect();

    let header = Row::new(vec![
        Cell::from("LABEL").style(Style::default().fg(theme::TEXT_MUTED)),
        Cell::from("ADDRESS").style(Style::default().fg(theme::TEXT_MUTED)),
        Cell::from("BALANCE").style(Style::default().fg(theme::TEXT_MUTED)),
        Cell::from("LAST ACTIVE").style(Style::default().fg(theme::TEXT_MUTED)),
    ])
    .height(1)
    .bottom_margin(1);

    let widths = [
        Constraint::Min(14),
        Constraint::Length(12),
        Constraint::Length(16),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(Line::from(vec![
                    Span::styled(
                        format!(" Tracking {} ", whale.wallets.len()),
                        Style::default().fg(theme::ACCENT),
                    ),
                    Span::styled(
                        " a:add  d:remove ",
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
    state.select(Some(whale.selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn render_activity_feed(frame: &mut Frame, area: Rect, app: &App) {
    let whale = &app.whale_state;

    let wallet = match whale.selected_wallet() {
        Some(w) => w,
        None => return,
    };

    let title = format!(" Activity: {} ", wallet.label);

    if wallet.loading {
        let loading = Paragraph::new("  Loading activity...")
            .style(Style::default().fg(theme::TEXT_MUTED))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title(title)
                    .title_style(Style::default().fg(theme::ACCENT)),
            );
        frame.render_widget(loading, area);
        return;
    }

    if wallet.recent_txs.is_empty() {
        let empty = Paragraph::new("  No recent activity")
            .style(Style::default().fg(theme::TEXT_MUTED))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title(title)
                    .title_style(Style::default().fg(theme::ACCENT)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let mut lines = Vec::new();

    for tx in wallet.recent_txs.iter().take(20) {
        let type_style = match tx.tx_type {
            TxType::Swap => Style::default().fg(theme::ACCENT),
            TxType::Transfer => Style::default().fg(theme::GREEN),
            _ => Style::default().fg(theme::TEXT_MUTED),
        };

        let detail = format_tx_detail(&tx.details);

        lines.push(Line::from(vec![
            Span::styled(format!("  {:<10}", tx.time_ago()), Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(format!("{:<10}", tx.tx_type.label()), type_style),
            Span::styled(detail, Style::default().fg(theme::TEXT_PRIMARY)),
        ]));
    }

    let feed = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(title)
            .title_style(Style::default().fg(theme::ACCENT)),
    );
    frame.render_widget(feed, area);
}

fn format_tx_detail(details: &TxDetails) -> String {
    match details {
        TxDetails::Swap {
            token_in_symbol,
            token_in_amount,
            token_out_symbol,
            token_out_amount,
        } => format!(
            "{:.2} {} -> {:.2} {}",
            token_in_amount, token_in_symbol, token_out_amount, token_out_symbol
        ),
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
            format!("{:.2} {} {} {}", amount, token_symbol, arrow, counterparty)
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
            format!("{:.4} SOL {} {}", amount_sol, arrow, counterparty)
        }
        TxDetails::Other { summary } => {
            if summary.len() > 40 {
                format!("{}...", &summary[..37])
            } else {
                summary.clone()
            }
        }
    }
}

fn short_addr(addr: &str) -> String {
    if addr.len() < 10 {
        return addr.to_string();
    }
    format!("{}...{}", &addr[..4], &addr[addr.len() - 4..])
}
