use super::{ClipboardComponent, Component, EventState, PropertyTrait, StatefulDrawableComponent};
use crate::clipboard::copy_to_clipboard;
use crate::components::command::{self, CommandInfo};
use crate::components::TableComponent;
use crate::config::KeyConfig;
use crate::database::Pool;
use crate::event::Key;
use crate::tree::{Database, Table};
use anyhow::Result;
use async_trait::async_trait;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

#[derive(Debug, PartialEq)]
pub enum Focus {
    Column,
    Constraint,
    ForeignKey,
    Index,
    Definition,
}

impl std::fmt::Display for Focus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct PropertiesComponent {
    column_table: TableComponent,
    constraint_table: TableComponent,
    foreign_key_table: TableComponent,
    index_table: TableComponent,
    definition_viewer: ClipboardComponent,
    focus: Focus,
    key_config: KeyConfig,
}

impl PropertiesComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            column_table: TableComponent::new(key_config.clone()),
            constraint_table: TableComponent::new(key_config.clone()),
            foreign_key_table: TableComponent::new(key_config.clone()),
            index_table: TableComponent::new(key_config.clone()),
            definition_viewer: ClipboardComponent::new(key_config.clone()),
            focus: Focus::Column,
            key_config,
        }
    }

    fn focused_component(&mut self) -> &mut dyn PropertyTrait {
        match self.focus {
            Focus::Column => &mut self.column_table,
            Focus::Constraint => &mut self.constraint_table,
            Focus::ForeignKey => &mut self.foreign_key_table,
            Focus::Index => &mut self.index_table,
            Focus::Definition => &mut self.definition_viewer,
        }
    }

    pub async fn update(
        &mut self,
        database: Database,
        table: Table,
        pool: &Box<dyn Pool>,
    ) -> Result<()> {
        self.column_table.reset();
        let columns = pool.get_columns(&database, &table).await?;
        if !columns.is_empty() {
            self.column_table.update(
                columns
                    .iter()
                    .map(|c| c.columns())
                    .collect::<Vec<Vec<String>>>(),
                None,
                columns.first().unwrap().fields(),
                database.clone(),
                table.clone(),
                false,
            );
        }
        self.constraint_table.reset();
        let constraints = pool.get_constraints(&database, &table).await?;
        if !constraints.is_empty() {
            self.constraint_table.update(
                constraints
                    .iter()
                    .map(|c| c.columns())
                    .collect::<Vec<Vec<String>>>(),
                None,
                constraints.first().unwrap().fields(),
                database.clone(),
                table.clone(),
                false,
            );
        }
        self.foreign_key_table.reset();
        let foreign_keys = pool.get_foreign_keys(&database, &table).await?;
        if !foreign_keys.is_empty() {
            self.foreign_key_table.update(
                foreign_keys
                    .iter()
                    .map(|c| c.columns())
                    .collect::<Vec<Vec<String>>>(),
                None,
                foreign_keys.first().unwrap().fields(),
                database.clone(),
                table.clone(),
                false,
            );
        }
        self.index_table.reset();
        let indexes = pool.get_indexes(&database, &table).await?;
        if !indexes.is_empty() {
            self.index_table.update(
                indexes
                    .iter()
                    .map(|c| c.columns())
                    .collect::<Vec<Vec<String>>>(),
                None,
                indexes.first().unwrap().fields(),
                database.clone(),
                table.clone(),
                false,
            );
        }
        // create table sql is here
        self.definition_viewer.reset();
        let definition = pool.get_definition(&database, &table).await?;
        if !definition.is_empty() {
            self.definition_viewer
                .update(definition, database.clone(), table.clone())
        }
        Ok(())
    }

    fn tab_names(&self) -> Vec<(Focus, String)> {
        vec![
            (Focus::Column, command::tab_columns(&self.key_config).name),
            (
                Focus::Constraint,
                command::tab_constraints(&self.key_config).name,
            ),
            (
                Focus::ForeignKey,
                command::tab_foreign_keys(&self.key_config).name,
            ),
            (Focus::Index, command::tab_indexes(&self.key_config).name),
            (
                Focus::Definition,
                command::tab_definition(&self.key_config).name,
            ),
        ]
    }
}

impl StatefulDrawableComponent for PropertiesComponent {
    fn draw(&mut self, f: &mut Frame, area: Rect, focused: bool) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(20), Constraint::Min(1)])
            .split(area);

        let tab_names = self
            .tab_names()
            .iter()
            .map(|(f, c)| {
                ListItem::new(c.to_string()).style(if *f == self.focus {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default()
                })
            })
            .collect::<Vec<ListItem>>();

        let tab_list = List::new(tab_names)
            .block(Block::default().borders(Borders::ALL).style(if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            }))
            .style(Style::default());

        f.render_widget(tab_list, layout[0]);

        self.focused_component().draw(f, layout[1], focused)?;
        Ok(())
    }
}

#[async_trait]
impl Component for PropertiesComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        out.push(CommandInfo::new(command::toggle_property_tabs(
            &self.key_config,
        )));
    }

    fn event(&mut self, key: Key) -> Result<EventState> {
        self.focused_component().event(key)?;

        if key == self.key_config.copy {
            if let Some(text) = self.focused_component().content() {
                copy_to_clipboard(text.as_str())?
            }
        } else if key == self.key_config.tab_columns {
            self.focus = Focus::Column;
        } else if key == self.key_config.tab_constraints {
            self.focus = Focus::Constraint;
        } else if key == self.key_config.tab_foreign_keys {
            self.focus = Focus::ForeignKey;
        } else if key == self.key_config.tab_indexes {
            self.focus = Focus::Index;
        } else if key == self.key_config.tab_definition {
            self.focus = Focus::Definition;
        }
        Ok(EventState::NotConsumed)
    }
}
