use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::Screen;
use crate::tui::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    wallet: &str,
    last_refresh: &str,
    loading: bool,
    screen: Screen,
) {
    let wallet_display = if wallet.len() >= 8 {
        format!("{}...{}", &wallet[..4], &wallet[wallet.len() - 4..])
    } else {
        wallet.to_string()
    };

    let div = Span::styled("  |  ", Style::default().fg(theme::BORDER));

    let mut spans = vec![
        Span::styled("  RPC: ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled("Helius", Style::default().fg(theme::GREEN)),
        div.clone(),
        Span::styled(wallet_display, Style::default().fg(theme::TEXT_SECONDARY)),
        div.clone(),
    ];

    if loading {
        spans.push(Span::styled(
            "refreshing...",
            Style::default().fg(theme::YELLOW),
        ));
    } else {
        spans.push(Span::styled(
            last_refresh.to_string(),
            Style::default().fg(theme::TEXT_MUTED),
        ));
    }

    spans.push(div);

    // Context-aware hotkeys per screen
    match screen {
        Screen::Portfolio => {
            hint(&mut spans, "j/k", "nav");
            hint(&mut spans, "s", "sort");
            hint(&mut spans, "y", "copy");
            hint(&mut spans, "r", "refresh");
        }
        Screen::Transactions => {
            hint(&mut spans, "j/k", "nav");
            hint(&mut spans, "y", "copy tx");
            hint(&mut spans, "r", "refresh");
        }
        Screen::Whales => {
            hint(&mut spans, "j/k", "nav");
            hint(&mut spans, "a", "add");
            hint(&mut spans, "d", "remove");
            hint(&mut spans, "y", "copy");
        }
        Screen::TokenLookup => {
            hint(&mut spans, "/", "search");
            hint(&mut spans, "y", "copy mint");
        }
        Screen::Help => {}
    }

    hint(&mut spans, "1-4", "tabs");
    hint(&mut spans, "?", "help");
    hint(&mut spans, "q", "quit");

    let status = Line::from(spans);
    let bar = Paragraph::new(status).style(Style::default().bg(theme::BG_SURFACE));
    frame.render_widget(bar, area);
}

fn hint(spans: &mut Vec<Span<'static>>, key: &'static str, desc: &'static str) {
    spans.push(Span::styled(key, Style::default().fg(theme::ACCENT)));
    spans.push(Span::styled(
        format!(":{desc}  "),
        Style::default().fg(theme::TEXT_MUTED),
    ));
}
