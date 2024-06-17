use super::{Component, EventState, StatefulDrawableComponent};
use crate::components::command::CommandInfo;
use crate::components::{TableComponent, TableFilterComponent};
use crate::config::KeyConfig;
use crate::event::Key;
use crate::tree::{Database, Table as DTable};
use anyhow::Result;
use ratatui::layout::Flex;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub enum Focus {
    Table,
    Filter,
}

pub struct RecordTableComponent {
    pub filter: TableFilterComponent,
    pub table: TableComponent,
    pub focus: Focus,
    key_config: KeyConfig,
}

impl RecordTableComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            filter: TableFilterComponent::new(key_config.clone()),
            table: TableComponent::new(key_config.clone()),
            focus: Focus::Table,
            key_config,
        }
    }

    pub fn update(
        &mut self,
        rows: Vec<Vec<String>>,
        total_row_count: Option<usize>,
        headers: Vec<String>,
        database: Database,
        table: DTable,
        hold_cursor_position: bool,
    ) {
        self.table.update(
            rows,
            total_row_count,
            headers,
            database,
            table.clone(),
            hold_cursor_position,
        );
        self.filter.table = Some(table);
    }

    pub fn reset(&mut self) {
        self.table.reset();
        self.filter.reset();
    }

    pub fn filter_focused(&self) -> bool {
        matches!(self.focus, Focus::Filter)
    }
}

impl StatefulDrawableComponent for RecordTableComponent {
    fn draw(&mut self, f: &mut Frame, area: Rect, focused: bool) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Length(5)])
            .flex(Flex::Legacy)
            .split(area);

        self.table
            .draw(f, layout[1], focused && matches!(self.focus, Focus::Table))?;

        self.filter
            .draw(f, layout[0], focused && matches!(self.focus, Focus::Filter))?;
        Ok(())
    }
}

impl Component for RecordTableComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        self.table.commands(out)
    }

    fn event(&mut self, key: Key) -> Result<EventState> {
        if key == self.key_config.filter {
            self.focus = Focus::Filter;
            return Ok(EventState::Consumed);
        }
        match key {
            key if matches!(self.focus, Focus::Filter) => return self.filter.event(key),
            key if matches!(self.focus, Focus::Table) => return self.table.event(key),
            _ => (),
        }
        Ok(EventState::NotConsumed)
    }
}
