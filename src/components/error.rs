use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::config::KeyConfig;
use crate::event::Key;
use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub struct ErrorComponent {
    pub error: String,
    visible: bool,
    key_config: KeyConfig,
}

impl ErrorComponent {
    const WIDTH: u16 = 65;
    const HEIGHT: u16 = 10;
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            error: String::new(),
            visible: false,
            key_config,
        }
    }

    pub fn set(&mut self, error: String) -> anyhow::Result<()> {
        self.error = error;
        self.show()
    }
}

impl DrawableComponent for ErrorComponent {
    fn draw(&self, f: &mut Frame, _area: Rect, _focused: bool) -> Result<()> {
        if self.visible {
            let error = Block::default()
                .title("Error")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Red));

            let area = Rect::new(
                (f.size().width.saturating_sub(Self::WIDTH)) / 2,
                (f.size().height.saturating_sub(Self::HEIGHT)) / 2,
                Self::WIDTH.min(f.size().width),
                Self::HEIGHT.min(f.size().height),
            );
            let chunks = Layout::default()
                .vertical_margin(1)
                .horizontal_margin(1)
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
                .split(area);

            f.render_widget(Clear, area);
            f.render_widget(error, area);
            f.render_widget(
                Paragraph::new(self.error.to_string()).wrap(Wrap { trim: true }),
                chunks[0],
            );
            f.render_widget(
                Paragraph::new(Line::from(vec![Span::styled(
                    format!(
                        "Press [{}] to close this modal.",
                        self.key_config.exit_popup
                    ),
                    Style::default(),
                )]))
                .alignment(Alignment::Right),
                chunks[1],
            );
        }
        Ok(())
    }
}

impl Component for ErrorComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        if self.visible {
            if key == self.key_config.exit_popup {
                self.error = String::new();
                self.hide();
                return Ok(EventState::Consumed);
            }
            return Ok(EventState::NotConsumed);
        }
        Ok(EventState::NotConsumed)
    }

    fn hide(&mut self) {
        self.visible = false;
    }

    fn show(&mut self) -> Result<()> {
        self.visible = true;

        Ok(())
    }
}
