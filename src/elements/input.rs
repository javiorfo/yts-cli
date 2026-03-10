use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::elements::Focus;

#[derive(Debug, Default)]
pub struct InputBox {
    pub text: String,
}

impl InputBox {
    pub fn render(&self, focus: &Focus) -> Paragraph<'_> {
        let style = if matches!(focus, Focus::InputBox) {
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(style)
            .title(" Search movie ");

        Paragraph::new(self.text.clone())
            .style(Style::default().fg(Color::Gray))
            .block(block)
    }
}
