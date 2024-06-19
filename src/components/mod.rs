pub mod clipboard;
pub mod command;
pub mod completion;
pub mod connections;
pub mod database_filter;
pub mod databases;
pub mod error;
pub mod help;
pub mod properties;
pub mod record_table;
pub mod sql_editor;
pub mod tab;
pub mod table;
pub mod table_filter;
pub mod table_status;
pub mod table_value;
pub mod utils;

#[cfg(debug_assertions)]
pub mod debug;
pub use clipboard::ClipboardComponent;
pub use command::CommandInfo;
pub use completion::CompletionComponent;
pub use connections::ConnectionsComponent;
pub use database_filter::DatabaseFilterComponent;
pub use databases::DatabasesComponent;
pub use error::ErrorComponent;
pub use help::HelpComponent;
pub use properties::PropertiesComponent;
pub use record_table::RecordTableComponent;
pub use sql_editor::SqlEditorComponent;
pub use tab::TabComponent;
pub use table::TableComponent;
pub use table_filter::TableFilterComponent;
pub use table_status::TableStatusComponent;
pub use table_value::TableValueComponent;

use crate::{database::Pool, event::Key};
use anyhow::Result;
use async_trait::async_trait;
use ratatui::{layout::Rect, Frame};
use std::convert::TryInto;
use unicode_width::UnicodeWidthChar;

#[derive(PartialEq, Debug)]
pub enum EventState {
    Consumed,
    NotConsumed,
}

impl EventState {
    pub fn is_consumed(&self) -> bool {
        *self == Self::Consumed
    }
}

impl From<bool> for EventState {
    fn from(consumed: bool) -> Self {
        if consumed {
            Self::Consumed
        } else {
            Self::NotConsumed
        }
    }
}

pub trait DrawableComponent {
    fn draw(&self, f: &mut Frame, rect: Rect, focused: bool) -> Result<()>;
}

pub trait StatefulDrawableComponent {
    fn draw(&mut self, f: &mut Frame, rect: Rect, focused: bool) -> Result<()>;
}

pub trait MovableComponent {
    fn draw(&mut self, f: &mut Frame, rect: Rect, focused: bool, x: u16, y: u16) -> Result<()>;
}

pub trait PropertyTrait {
    fn draw(&mut self, f: &mut Frame, rect: Rect, focused: bool) -> Result<()>;
    fn event(&mut self, key: Key) -> Result<EventState>;
    fn selected_cells(&self) -> Option<String>;
}

/// base component trait
#[async_trait]
pub trait Component {
    fn commands(&self, out: &mut Vec<CommandInfo>);

    fn event(&mut self, key: crate::event::Key) -> Result<EventState>;

    async fn async_event(
        &mut self,
        _key: crate::event::Key,
        _pool: &Box<dyn Pool>,
    ) -> Result<EventState> {
        Ok(EventState::NotConsumed)
    }

    fn hide(&mut self) {}

    fn show(&mut self) -> Result<()> {
        Ok(())
    }
}

fn compute_character_width(c: char) -> u16 {
    UnicodeWidthChar::width(c).unwrap().try_into().unwrap()
}
