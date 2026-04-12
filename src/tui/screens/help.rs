use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::theme;

pub fn render(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  NAVIGATION", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        help_line("1-4", "Switch tabs"),
        help_line("j / Down", "Move down"),
        help_line("k / Up", "Move up"),
        help_line("g", "Jump to top"),
        help_line("G", "Jump to bottom"),
        help_line("r", "Refresh data"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  GENERAL", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        help_line("?", "Toggle help"),
        help_line("q / Ctrl+C", "Quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  PORTFOLIO", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        help_line("s", "Cycle sort (value/name/balance)"),
        help_line("y", "Copy selected address"),
    ];

    let help = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER))
            .title(" Keybindings ")
            .title_style(Style::default().fg(theme::ACCENT)),
    );

    frame.render_widget(help, area);
}

fn help_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {:<14}", key), Style::default().fg(theme::ACCENT)),
        Span::styled(desc, Style::default().fg(theme::TEXT_SECONDARY)),
    ])
}
