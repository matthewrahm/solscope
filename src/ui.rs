use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Tabs,
    Frame,
};

use crate::app::{App, Screen, TABS};
use crate::tui::{screens, theme, widgets::status_bar};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(2), // tabs
        Constraint::Min(0),    // content
        Constraint::Length(1), // status bar
    ])
    .split(frame.area());

    // Clear background
    frame.render_widget(
        ratatui::widgets::Block::default().style(Style::default().bg(theme::BG_BASE)),
        frame.area(),
    );

    render_tabs(frame, chunks[0], app);
    render_content(frame, chunks[1], app);
    // Status message overrides status bar briefly
    if let Some(msg) = app.status_message() {
        let bar = ratatui::widgets::Paragraph::new(format!("  {msg}"))
            .style(Style::default().fg(theme::GREEN).bg(theme::BG_SURFACE));
        frame.render_widget(bar, chunks[2]);
    } else {
        status_bar::render(
            frame,
            chunks[2],
            &app.wallet,
            &app.last_refresh_label(),
            app.loading,
            app.screen,
        );
    }
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line> = TABS
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let num = format!("{}", i + 1);
            let label = tab.label();
            Line::from(vec![
                Span::styled(format!(" {num}"), Style::default().fg(theme::ACCENT)),
                Span::styled(
                    format!(" {label} "),
                    if app.screen == *tab {
                        Style::default()
                            .fg(theme::TEXT_PRIMARY)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme::TEXT_MUTED)
                    },
                ),
            ])
        })
        .collect();

    let selected = TABS.iter().position(|t| *t == app.screen).unwrap_or(0);

    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled(" | ", Style::default().fg(theme::BORDER)));

    frame.render_widget(tabs, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.screen {
        Screen::Portfolio => screens::portfolio::render(frame, area, app),
        Screen::Transactions => screens::transactions::render(frame, area, app),
        Screen::TokenLookup => screens::token_lookup::render(frame, area, app),
        Screen::Help => screens::help::render(frame, area),
        Screen::Whales => screens::whales::render(frame, area, app),
    }
}
