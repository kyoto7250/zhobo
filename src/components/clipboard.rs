use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{
    config::KeyConfig,
    event::Key,
    tree::{Database, Table as DTable},
};

use super::{utils::scroll_vertical::VerticalScroll, EventState, PropertyTrait};

pub struct ClipboardComponent {
    table: Option<(Database, DTable)>,
    content: Option<String>,
    key_config: KeyConfig,
    position: u16,
    scroll: VerticalScroll,
}

impl ClipboardComponent {
    const MARGIN: u16 = 1;
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            table: None,
            content: None,
            key_config,
            position: 0,
            scroll: VerticalScroll::new(false, false),
        }
    }

    pub fn reset(&mut self) {
        self.table = None;
        self.content = None;
        self.position = 0;
    }

    pub fn title(&mut self) -> String {
        self.table.as_ref().map_or(" - ".to_string(), |table| {
            format!("{}.{}", table.0.name, table.1.name)
        })
    }

    pub fn update(&mut self, content: String, database: Database, table: DTable) {
        self.content = Some(content);
        self.table = Some((database, table));
    }

    pub fn unwrap_content(&self) -> String {
        self.content.clone().unwrap_or(String::from(""))
    }
}

impl PropertyTrait for ClipboardComponent {
    fn draw(&mut self, f: &mut Frame, rect: Rect, focused: bool) -> anyhow::Result<()> {
        f.render_widget(
            Block::default()
                .title(self.title())
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .style(if focused {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
            rect,
        );

        let chunks = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(1)
            .direction(Direction::Vertical)
            .flex(Flex::Legacy)
            .constraints([Constraint::Min(1)].as_ref())
            .split(rect.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }));

        // can scroll = content.height - widget.height
        let paragraph = Paragraph::new(self.unwrap_content())
            .scroll((self.position, 0))
            .wrap(Wrap { trim: false });

        let content_height = paragraph.line_count(chunks[0].width);
        let rect_height = (chunks[0].height - Self::MARGIN) as usize;
        let diff = (content_height).saturating_sub(rect_height);
        self.position = std::cmp::min(self.position, diff as u16);

        self.scroll.update(
            self.position as usize,
            content_height,
            chunks[0].height.saturating_sub(2) as usize,
        );
        self.scroll.draw(f, chunks[0]);

        f.render_widget(paragraph, chunks[0]);

        Ok(())
    }

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.key_config.scroll_down {
            self.position += 1;
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_up {
            self.position = self.position.saturating_sub(1);
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }

    fn content(&self) -> Option<String> {
        self.content.clone()
    }
}
