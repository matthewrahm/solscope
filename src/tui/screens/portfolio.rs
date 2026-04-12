use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::app::App;
use crate::tui::{theme, widgets::token_table};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(5), // summary header
        Constraint::Min(10),  // token table
    ])
    .split(area);

    render_summary(frame, chunks[0], app);
    render_holdings(frame, chunks[1], app);
}

fn render_summary(frame: &mut Frame, area: Rect, app: &App) {
    let portfolio = match &app.portfolio {
        Some(p) => p,
        None => {
            let loading = Paragraph::new("  Loading portfolio...")
                .style(Style::default().fg(theme::TEXT_MUTED));
            frame.render_widget(loading, area);
            return;
        }
    };

    let cols = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
    ])
    .split(area);

    // Left: wallet + total value
    let total = format_usd(portfolio.total_value);
    let left = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Total Value  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(
                &total,
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  SOL Balance  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(
                format!("{:.4} SOL", portfolio.sol_balance),
                Style::default().fg(theme::ACCENT),
            ),
            Span::styled(
                format!("  ({})", format_usd(portfolio.sol_value)),
                Style::default().fg(theme::TEXT_SECONDARY),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(" Overview ")
            .title_style(Style::default().fg(theme::ACCENT)),
    );
    frame.render_widget(left, cols[0]);

    // Middle: quick stats
    let token_count = portfolio
        .holdings
        .iter()
        .filter(|h| h.value_usd >= 0.01 || h.price_usd > 0.0)
        .count();
    let sol_pct = if portfolio.total_value > 0.0 {
        (portfolio.sol_value / portfolio.total_value * 100.0) as u32
    } else {
        0
    };

    let sort_label = match app.sort_mode {
        crate::app::SortMode::Value => "value",
        crate::app::SortMode::Name => "name",
        crate::app::SortMode::Balance => "balance",
    };

    let mid = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tokens  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(format!("{token_count}"), Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled(
                format!("    Sort: {sort_label}"),
                Style::default().fg(theme::TEXT_MUTED),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  SOL %   ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(format!("{sol_pct}%"), Style::default().fg(theme::ACCENT)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(" Stats ")
            .title_style(Style::default().fg(theme::ACCENT)),
    );
    frame.render_widget(mid, cols[1]);

    // Right: SOL price sparkline
    let sol_mint = "So11111111111111111111111111111111111111112";
    let sparkline_data: Vec<u64> = app
        .price_history
        .get(sol_mint)
        .map(|prices| {
            // Normalize prices to 0-100 range for sparkline
            let min = prices.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let range = (max - min).max(0.001);
            prices
                .iter()
                .map(|&p| ((p - min) / range * 100.0) as u64)
                .collect()
        })
        .unwrap_or_default();

    let price_label = format!("${:.2}", portfolio.sol_price);

    if sparkline_data.len() >= 2 {
        let spark = Sparkline::default()
            .data(&sparkline_data)
            .bar_set(symbols::bar::NINE_LEVELS)
            .style(Style::default().fg(theme::ACCENT))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title(format!(" SOL {price_label} "))
                    .title_style(Style::default().fg(theme::ACCENT)),
            );
        frame.render_widget(spark, cols[2]);
    } else {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!("  SOL {price_label}"),
                    Style::default().fg(theme::ACCENT),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  collecting data...",
                    Style::default().fg(theme::TEXT_MUTED),
                ),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(" SOL Price ")
                .title_style(Style::default().fg(theme::ACCENT)),
        );
        frame.render_widget(placeholder, cols[2]);
    }
}

fn render_holdings(frame: &mut Frame, area: Rect, app: &App) {
    let portfolio = match &app.portfolio {
        Some(p) => p,
        None => return,
    };

    let (table, mut state) = token_table::render_token_table(
        &portfolio.holdings,
        portfolio.sol_balance,
        portfolio.sol_price,
        app.table_selected,
    );
    frame.render_stateful_widget(table, area, &mut state);
}

fn format_usd(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("${:.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.2}", value)
    } else {
        format!("${:.2}", value)
    }
}
