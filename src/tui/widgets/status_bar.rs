use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::theme;

pub fn render(frame: &mut Frame, area: Rect, wallet: &str, last_refresh: &str, loading: bool) {
    let wallet_display = if wallet.len() >= 8 {
        format!("{}...{}", &wallet[..4], &wallet[wallet.len() - 4..])
    } else {
        wallet.to_string()
    };

    let mut spans = vec![
        Span::styled("  RPC: ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("Helius", Style::default().fg(theme::GREEN)),
        Span::styled("  |  ", Style::default().fg(theme::BORDER)),
        Span::styled(wallet_display, Style::default().fg(theme::TEXT_SECONDARY)),
        Span::styled("  |  ", Style::default().fg(theme::BORDER)),
    ];

    if loading {
        spans.push(Span::styled(
            "refreshing...",
            Style::default().fg(theme::YELLOW),
        ));
    } else {
        spans.push(Span::styled(
            last_refresh,
            Style::default().fg(theme::TEXT_MUTED),
        ));
    }

    spans.extend([
        Span::styled("  |  ", Style::default().fg(theme::BORDER)),
        Span::styled("j/k", Style::default().fg(theme::ACCENT)),
        Span::styled(":nav  ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("1-4", Style::default().fg(theme::ACCENT)),
        Span::styled(":tabs  ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("?", Style::default().fg(theme::ACCENT)),
        Span::styled(":help  ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("q", Style::default().fg(theme::ACCENT)),
        Span::styled(":quit", Style::default().fg(theme::TEXT_MUTED)),
    ]);

    let status = Line::from(spans);
    let bar = Paragraph::new(status).style(Style::default().bg(theme::BG_SURFACE));
    frame.render_widget(bar, area);
}
