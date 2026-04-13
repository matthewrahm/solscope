use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::tui::{theme, widgets::price_ticker};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // search input
        Constraint::Length(4), // live price ticker
        Constraint::Min(0),    // results
    ])
    .split(area);

    render_search_input(frame, chunks[0], app);

    if let Some(info) = &app.token_info {
        let ticker_data = price_ticker::PriceTickerData {
            symbol: &info.symbol,
            price: info.price_usd,
            change_1h: info.price_change_1h,
            change_24h: info.price_change_24h,
            history: app.price_history.get(&info.mint),
        };
        price_ticker::render(frame, chunks[1], &ticker_data);
        render_details(frame, chunks[2], app);
    } else if app.token_loading {
        let loading = Paragraph::new("  Fetching token data...")
            .style(Style::default().fg(theme::TEXT_MUTED));
        // Merge chunks[1] and chunks[2] visually
        let merged = Rect {
            y: chunks[1].y,
            height: chunks[1].height + chunks[2].height,
            ..chunks[1]
        };
        frame.render_widget(loading, merged);
    } else {
        let hint = Paragraph::new(
            "  Enter a Solana token mint address to look up market data and security info",
        )
        .style(Style::default().fg(theme::TEXT_MUTED));
        let merged = Rect {
            y: chunks[1].y,
            height: chunks[1].height + chunks[2].height,
            ..chunks[1]
        };
        frame.render_widget(hint, merged);
    }
}

fn render_search_input(frame: &mut Frame, area: Rect, app: &App) {
    let display_text = if app.token_input_active {
        format!("{}|", &app.token_search_input)
    } else if app.token_search_input.is_empty() {
        "Press / to enter a token mint address...".to_string()
    } else {
        app.token_search_input.clone()
    };

    let style = if app.token_input_active {
        Style::default().fg(theme::TEXT_PRIMARY)
    } else {
        Style::default().fg(theme::TEXT_MUTED)
    };

    let input = Paragraph::new(format!("  {display_text}"))
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if app.token_input_active {
                    Style::default().fg(theme::ACCENT)
                } else {
                    Style::default().fg(theme::BORDER)
                })
                .title(" Search Token ")
                .title_style(Style::default().fg(theme::ACCENT)),
        );

    frame.render_widget(input, area);
}

fn render_details(frame: &mut Frame, area: Rect, app: &App) {
    let info = match &app.token_info {
        Some(info) => info,
        None => return,
    };

    let cols =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    // Left: market data
    let mcap_str = format_big_number(info.market_cap);
    let fdv_str = format_big_number(info.fdv);
    let vol_str = format_big_number(info.volume_24h);
    let liq_str = format_big_number(info.liquidity);

    let mut market_lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {} ({})", info.name, info.symbol),
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        stat_line("MCap", &mcap_str),
        stat_line("FDV", &fdv_str),
        stat_line("Vol 24h", &vol_str),
        stat_line("Liquidity", &liq_str),
        Line::from(""),
        stat_line("DEX", &info.dex),
    ];

    if !info.mint.is_empty() {
        market_lines.push(Line::from(""));
        market_lines.push(Line::from(vec![
            Span::styled("  Mint  ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(&info.mint, Style::default().fg(theme::TEXT_SECONDARY)),
        ]));
    }

    let market = Paragraph::new(market_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(" Market Data ")
            .title_style(Style::default().fg(theme::ACCENT)),
    );
    frame.render_widget(market, cols[0]);

    // Right: security
    let mut sec_lines = vec![Line::from("")];

    if let Some(sec) = &info.security {
        let risk_color = match sec.risk_level.as_str() {
            "LOW" => theme::GREEN,
            "MEDIUM" => theme::YELLOW,
            "HIGH" => theme::RED,
            _ => theme::TEXT_MUTED,
        };

        sec_lines.push(Line::from(vec![
            Span::styled("  Risk      ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(
                &sec.risk_level,
                Style::default().fg(risk_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  ({})",
                    sec.score
                        .map(|s| format!("{:.0}", s))
                        .unwrap_or_else(|| "?".to_string())
                ),
                Style::default().fg(theme::TEXT_SECONDARY),
            ),
        ]));
        sec_lines.push(Line::from(""));

        sec_lines.push(check_line(
            "Mint Auth",
            sec.mint_revoked,
            "Revoked",
            "ACTIVE",
        ));
        sec_lines.push(check_line(
            "Freeze",
            sec.freeze_revoked,
            "Revoked",
            "ACTIVE",
        ));
        sec_lines.push(Line::from(""));
        sec_lines.push(Line::from(vec![
            Span::styled("  Top 10    ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(
                format!("{:.1}%", sec.top_10_pct),
                Style::default().fg(if sec.top_10_pct > 50.0 {
                    theme::RED
                } else if sec.top_10_pct > 30.0 {
                    theme::YELLOW
                } else {
                    theme::GREEN
                }),
            ),
        ]));

        if !sec.risks.is_empty() {
            sec_lines.push(Line::from(""));
            sec_lines.push(Line::from(vec![Span::styled(
                "  Risks",
                Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
            )]));
            for risk in &sec.risks {
                let truncated = if risk.len() > 40 {
                    format!("  {}...", &risk[..37])
                } else {
                    format!("  {risk}")
                };
                sec_lines.push(Line::from(vec![Span::styled(
                    truncated,
                    Style::default().fg(theme::TEXT_SECONDARY),
                )]));
            }
        }
    } else {
        sec_lines.push(Line::from(vec![Span::styled(
            "  Security data unavailable",
            Style::default().fg(theme::TEXT_MUTED),
        )]));
    }

    let security = Paragraph::new(sec_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(" Security ")
            .title_style(Style::default().fg(theme::ACCENT)),
    );
    frame.render_widget(security, cols[1]);
}

fn stat_line<'a>(label: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:<10}", label),
            Style::default().fg(theme::TEXT_MUTED),
        ),
        Span::styled(value.to_string(), Style::default().fg(theme::TEXT_PRIMARY)),
    ])
}

fn check_line<'a>(label: &'a str, ok: bool, ok_text: &'a str, bad_text: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:<10}", label),
            Style::default().fg(theme::TEXT_MUTED),
        ),
        Span::styled(
            if ok { ok_text } else { bad_text },
            Style::default().fg(if ok { theme::GREEN } else { theme::RED }),
        ),
    ])
}

fn format_big_number(value: Option<f64>) -> String {
    match value {
        Some(v) if v >= 1_000_000_000.0 => format!("${:.2}B", v / 1_000_000_000.0),
        Some(v) if v >= 1_000_000.0 => format!("${:.2}M", v / 1_000_000.0),
        Some(v) if v >= 1_000.0 => format!("${:.2}K", v / 1_000.0),
        Some(v) => format!("${:.2}", v),
        None => "-".to_string(),
    }
}
