use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::tui::theme;

pub struct PriceTickerData<'a> {
    pub symbol: &'a str,
    pub price: f64,
    pub change_1h: Option<f64>,
    pub change_24h: Option<f64>,
    pub history: Option<&'a [f64]>,
}

pub fn render(frame: &mut Frame, area: Rect, data: &PriceTickerData) {
    let cols = Layout::horizontal([
        Constraint::Min(20),    // price + change
        Constraint::Length(20), // sparkline
    ])
    .split(area);

    // Left: price and changes
    let price_str = format_price(data.price);
    let change_1h = format_change(data.change_1h);
    let change_24h = format_change(data.change_24h);

    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!("  {} ", data.symbol),
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                price_str,
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  1h ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(change_1h.0, Style::default().fg(change_1h.1)),
            Span::styled("  24h ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(change_24h.0, Style::default().fg(change_24h.1)),
        ]),
    ];

    let price_widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(" Live Price ")
            .title_style(Style::default().fg(theme::ACCENT)),
    );
    frame.render_widget(price_widget, cols[0]);

    // Right: sparkline
    let sparkline_data: Vec<u64> = data
        .history
        .filter(|h| h.len() >= 2)
        .map(|prices| {
            let min = prices.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let range = (max - min).max(0.000001);
            prices
                .iter()
                .map(|&p| ((p - min) / range * 100.0) as u64)
                .collect()
        })
        .unwrap_or_default();

    if sparkline_data.len() >= 2 {
        // Determine color based on trend
        let trend_color = if sparkline_data.last() > sparkline_data.first() {
            theme::GREEN
        } else if sparkline_data.last() < sparkline_data.first() {
            theme::RED
        } else {
            theme::ACCENT
        };

        let spark = Sparkline::default()
            .data(&sparkline_data)
            .bar_set(symbols::bar::NINE_LEVELS)
            .style(Style::default().fg(trend_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER)),
            );
        frame.render_widget(spark, cols[1]);
    } else {
        let placeholder = Paragraph::new(Line::from(vec![Span::styled(
            " waiting...",
            Style::default().fg(theme::TEXT_MUTED),
        )]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER)),
        );
        frame.render_widget(placeholder, cols[1]);
    }
}

fn format_price(price: f64) -> String {
    if price == 0.0 {
        "-".to_string()
    } else if price >= 1.0 {
        format!("${:.2}", price)
    } else if price >= 0.01 {
        format!("${:.4}", price)
    } else if price >= 0.0001 {
        format!("${:.6}", price)
    } else {
        format!("${:.10}", price)
    }
}

fn format_change(value: Option<f64>) -> (String, ratatui::style::Color) {
    match value {
        Some(v) if v >= 0.0 => (format!("+{:.2}%", v), theme::GREEN),
        Some(v) => (format!("{:.2}%", v), theme::RED),
        None => ("-".to_string(), theme::TEXT_MUTED),
    }
}
