use crate::clipboard::copy_to_clipboard;
use crate::components::{
    tab::Tab,
    {
        command, ConnectionsComponent, DatabasesComponent, ErrorComponent, HelpComponent,
        PropertiesComponent, RecordTableComponent, SqlEditorComponent, TabComponent,
    },
};
use crate::components::{
    CommandInfo, Component as _, DrawableComponent as _, EventState, StatefulDrawableComponent,
};
use crate::config::Config;
use crate::connection::{default_limit_size, Connection};
use crate::database::{MySqlPool, Pool, PostgresPool, SqlitePool};
use crate::event::Key;
use anyhow::Context;
use ratatui::layout::Flex;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub enum Focus {
    DatabaseList,
    Table,
    ConnectionList,
}
pub struct App {
    record_table: RecordTableComponent,
    properties: PropertiesComponent,
    sql_editor: SqlEditorComponent,
    focus: Focus,
    tab: TabComponent,
    help: HelpComponent,
    databases: DatabasesComponent,
    connections: ConnectionsComponent,
    pool: Option<Box<dyn Pool>>,
    left_main_chunk_percentage: u16,
    pub config: Config,
    pub error: ErrorComponent,
}

impl App {
    pub fn new(config: Config) -> App {
        Self {
            config: config.clone(),
            connections: ConnectionsComponent::new(config.key_config.clone(), config.conn),
            record_table: RecordTableComponent::new(config.key_config.clone()),
            properties: PropertiesComponent::new(config.key_config.clone()),
            sql_editor: SqlEditorComponent::new(config.key_config.clone()),
            tab: TabComponent::new(config.key_config.clone()),
            help: HelpComponent::new(config.key_config.clone()),
            databases: DatabasesComponent::new(config.key_config.clone()),
            error: ErrorComponent::new(config.key_config),
            focus: Focus::ConnectionList,
            pool: None,
            left_main_chunk_percentage: 15,
        }
    }

    pub fn draw(&mut self, f: &mut Frame) -> anyhow::Result<()> {
        if let Focus::ConnectionList = self.focus {
            match self.connections.draw(
                f,
                Layout::default()
                    .constraints([Constraint::Percentage(100)])
                    .split(f.size())[0],
                false,
            ) {
                Ok(()) => (),
                Err(e) => {
                    return Err(anyhow::anyhow!(e).context("from: ConnectionsComponent::draw"));
                }
            }

            self.error.draw(f, Rect::default(), false)?;
            self.help.draw(f, Rect::default(), false)?;
            return Ok(());
        }

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.left_main_chunk_percentage),
                Constraint::Percentage((100_u16).saturating_sub(self.left_main_chunk_percentage)),
            ])
            .split(f.size());

        self.databases
            .draw(f, main_chunks[0], matches!(self.focus, Focus::DatabaseList))?;

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Legacy)
            .constraints([Constraint::Length(3), Constraint::Length(5)].as_ref())
            .split(main_chunks[1]);

        self.tab.draw(f, right_chunks[0], false)?;

        match self.tab.selected_tab {
            Tab::Records => {
                self.record_table
                    .draw(f, right_chunks[1], matches!(self.focus, Focus::Table))?
            }
            Tab::Sql => {
                self.sql_editor
                    .draw(f, right_chunks[1], matches!(self.focus, Focus::Table))?;
            }
            Tab::Properties => {
                self.properties
                    .draw(f, right_chunks[1], matches!(self.focus, Focus::Table))?;
            }
        }
        self.error.draw(f, Rect::default(), false)?;
        self.help.draw(f, Rect::default(), false)?;
        Ok(())
    }

    fn update_commands(&mut self) {
        self.help.set_cmds(self.commands());
    }

    fn commands(&self) -> Vec<CommandInfo> {
        let mut res = vec![
            CommandInfo::new(command::exit_pop_up(&self.config.key_config)),
            CommandInfo::new(command::filter(&self.config.key_config)),
            CommandInfo::new(command::help(&self.config.key_config)),
            CommandInfo::new(command::toggle_tabs(&self.config.key_config)),
            CommandInfo::new(command::scroll(&self.config.key_config)),
            CommandInfo::new(command::scroll_to_top_bottom(&self.config.key_config)),
            CommandInfo::new(command::scroll_up_down_multiple_lines(
                &self.config.key_config,
            )),
            CommandInfo::new(command::move_focus(&self.config.key_config)),
            CommandInfo::new(command::extend_or_shorten_widget_width(
                &self.config.key_config,
            )),
        ];

        self.databases.commands(&mut res);
        self.record_table.commands(&mut res);
        self.properties.commands(&mut res);

        res
    }

    async fn update_databases(&mut self) -> anyhow::Result<()> {
        if let Some(conn) = self.connections.selected_connection() {
            if let Some(pool) = self.pool.as_ref() {
                pool.close().await;
            }

            match conn.database_url() {
                Ok(url) => {
                    self.pool = match conn {
                        Connection::MySql(conn) => Some(Box::new(
                            MySqlPool::new(url.as_str(), conn.limit_size, conn.timeout_second)
                                .await?,
                        )),
                        Connection::Postgres(conn) => Some(Box::new(
                            PostgresPool::new(url.as_str(), conn.limit_size, conn.timeout_second)
                                .await?,
                        )),
                        Connection::Sqlite(conn) => Some(Box::new(
                            SqlitePool::new(url.as_str(), conn.limit_size, conn.timeout_second)
                                .await?,
                        )),
                    };
                    self.databases
                        .update(conn, self.pool.as_ref().unwrap())
                        .await?;
                    self.focus = Focus::DatabaseList;
                    self.record_table.reset();
                    self.tab.reset();
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(e)).context("from Connection::database_url");
                }
            }
        }
        Ok(())
    }

    async fn update_record_table(&mut self, hold_cursor_position: bool) -> anyhow::Result<()> {
        if let Some((database, table)) = self.databases.tree().selected_table() {
            let order_query = self.record_table.table.generate_order_query();
            let (headers, records) = self
                .pool
                .as_ref()
                .unwrap()
                .get_records(
                    &database,
                    &table,
                    0,
                    if self.record_table.filter.input_str().is_empty() {
                        None
                    } else {
                        Some(self.record_table.filter.input_str())
                    },
                    order_query,
                )
                .await?;
            let total_row_count = self
                .pool
                .as_ref()
                .unwrap()
                .get_total_row_count(
                    &database,
                    &table,
                    if self.record_table.filter.input_str().is_empty() {
                        None
                    } else {
                        Some(self.record_table.filter.input_str())
                    },
                )
                .await?;

            let header_icons = self.record_table.table.generate_header_icons(headers.len());
            self.record_table.update(
                records,
                Some(total_row_count),
                self.concat_headers(headers, Some(header_icons)),
                database.clone(),
                table.clone(),
                hold_cursor_position,
            );
        }
        Ok(())
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        self.update_commands();

        if self.components_event(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        if self.move_focus(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        };
        Ok(EventState::NotConsumed)
    }

    async fn components_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if self.error.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        if !matches!(self.focus, Focus::ConnectionList) && self.help.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        match self.focus {
            Focus::ConnectionList => {
                if self.connections.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.enter {
                    self.update_databases().await?;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::DatabaseList => {
                if self.databases.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.enter && self.databases.tree_focused() {
                    if let Some((database, table)) = self.databases.tree().selected_table() {
                        self.record_table.reset();
                        let (headers, records) = self
                            .pool
                            .as_ref()
                            .unwrap()
                            .get_records(&database, &table, 0, None, None)
                            .await?;
                        let total_row_count = self
                            .pool
                            .as_ref()
                            .unwrap()
                            .get_total_row_count(&database, &table, None)
                            .await?;
                        self.record_table.update(
                            records,
                            Some(total_row_count),
                            headers,
                            database.clone(),
                            table.clone(),
                            false,
                        );
                        self.properties
                            .update(database.clone(), table.clone(), self.pool.as_ref().unwrap())
                            .await?;
                        self.focus = Focus::Table;
                    }
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Table => {
                match self.tab.selected_tab {
                    Tab::Records => {
                        if self.record_table.event(key)?.is_consumed() {
                            return Ok(EventState::Consumed);
                        };

                        if key == self.config.key_config.sort_by_column
                            && !self.record_table.table.headers.is_empty()
                        {
                            self.record_table.table.add_order();
                            self.update_record_table(true).await?;
                            return Ok(EventState::Consumed);
                        };

                        if key == self.config.key_config.copy {
                            if let Some(text) = self.record_table.table.content() {
                                copy_to_clipboard(text.as_str())?
                            }
                        }

                        if key == self.config.key_config.enter && self.record_table.filter_focused()
                        {
                            self.record_table.focus = crate::components::record_table::Focus::Table;
                            self.update_record_table(false).await?;
                        }

                        if self.record_table.table.eod {
                            return Ok(EventState::Consumed);
                        }

                        if let Some(index) = self.record_table.table.selected_row.selected() {
                            let limit_size =
                                if let Some(connection) = self.connections.selected_connection() {
                                    match connection {
                                        Connection::MySql(conn) => conn.limit_size,
                                        Connection::Postgres(conn) => conn.limit_size,
                                        Connection::Sqlite(conn) => conn.limit_size,
                                    }
                                } else {
                                    default_limit_size()
                                };
                            if index.saturating_add(1) % limit_size == 0
                                && index >= self.record_table.table.rows.len() - 1
                            {
                                if let Some((database, table)) =
                                    self.databases.tree().selected_table()
                                {
                                    let (_, records) = self
                                        .pool
                                        .as_ref()
                                        .unwrap()
                                        .get_records(
                                            &database,
                                            &table,
                                            index.saturating_add(1) as u16,
                                            if self.record_table.filter.input_str().is_empty() {
                                                None
                                            } else {
                                                Some(self.record_table.filter.input_str())
                                            },
                                            None,
                                        )
                                        .await?;
                                    if !records.is_empty() {
                                        self.record_table.table.rows.extend(records);
                                    } else {
                                        self.record_table.table.end()
                                    }
                                }
                            }
                        };
                    }
                    Tab::Sql => {
                        if self.sql_editor.event(key)?.is_consumed()
                            || self
                                .sql_editor
                                .async_event(key, self.pool.as_ref().unwrap())
                                .await?
                                .is_consumed()
                        {
                            return Ok(EventState::Consumed);
                        };
                    }
                    Tab::Properties => {
                        if self.properties.event(key)?.is_consumed() {
                            return Ok(EventState::Consumed);
                        };
                    }
                };
            }
        }

        if self.extend_or_shorten_widget_width(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        Ok(EventState::NotConsumed)
    }

    fn concat_headers(
        &self,
        headers: Vec<String>,
        header_icons: Option<Vec<String>>,
    ) -> Vec<String> {
        // If there are no execution results, there is no header, so do not combine icons.
        // This logic will not work properly if the number of columns changes in the future release.
        if headers.is_empty() {
            return headers;
        }
        if let Some(header_icons) = &header_icons {
            let mut new_headers = vec![String::new(); headers.len()];
            for (index, header) in headers.iter().enumerate() {
                // It does not support increasing or decreasing table columns using filter.
                // Rewrite when implementing column deletion
                new_headers[index] = format!(
                    "{} {}",
                    header,
                    header_icons.get(index).unwrap_or(&String::from(""))
                )
                .trim()
                .to_string();
            }
            return new_headers;
        }

        headers
    }

    fn extend_or_shorten_widget_width(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key
            == self
                .config
                .key_config
                .extend_or_shorten_widget_width_to_left
        {
            self.left_main_chunk_percentage =
                self.left_main_chunk_percentage.saturating_sub(5).max(15);
            return Ok(EventState::Consumed);
        } else if key
            == self
                .config
                .key_config
                .extend_or_shorten_widget_width_to_right
        {
            self.left_main_chunk_percentage = (self.left_main_chunk_percentage + 5).min(70);
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }

    fn move_focus(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.config.key_config.focus_connections {
            self.focus = Focus::ConnectionList;
            return Ok(EventState::Consumed);
        }
        if self.tab.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }
        match self.focus {
            Focus::ConnectionList => {
                if key == self.config.key_config.enter {
                    self.focus = Focus::DatabaseList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::DatabaseList => {
                if key == self.config.key_config.focus_right && self.databases.tree_focused() {
                    self.focus = Focus::Table;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Table => {
                if key == self.config.key_config.focus_left {
                    self.focus = Focus::DatabaseList;
                    return Ok(EventState::Consumed);
                }
            }
        }
        Ok(EventState::NotConsumed)
    }
}

#[cfg(test)]
mod test {
    use super::{App, Config, EventState, Key};

    #[test]
    fn test_extend_or_shorten_widget_width() {
        let mut app = App::new(Config::default());
        assert_eq!(
            app.extend_or_shorten_widget_width(Key::Char('>')).unwrap(),
            EventState::Consumed
        );
        assert_eq!(app.left_main_chunk_percentage, 20);

        app.left_main_chunk_percentage = 70;
        assert_eq!(
            app.extend_or_shorten_widget_width(Key::Char('>')).unwrap(),
            EventState::Consumed
        );
        assert_eq!(app.left_main_chunk_percentage, 70);

        assert_eq!(
            app.extend_or_shorten_widget_width(Key::Char('<')).unwrap(),
            EventState::Consumed
        );
        assert_eq!(app.left_main_chunk_percentage, 65);

        app.left_main_chunk_percentage = 15;
        assert_eq!(
            app.extend_or_shorten_widget_width(Key::Char('<')).unwrap(),
            EventState::Consumed
        );
        assert_eq!(app.left_main_chunk_percentage, 15);
    }

    #[test]
    fn test_concat_headers() {
        let app = App::new(Config::default());
        let headers = vec![
            "ID".to_string(),
            "NAME".to_string(),
            "TIMESTAMP".to_string(),
        ];
        let header_icons = vec!["".to_string(), "↑1".to_string(), "↓2".to_string()];
        let concat_headers: Vec<String> = app.concat_headers(headers, Some(header_icons));

        assert_eq!(
            concat_headers,
            vec![
                "ID".to_string(),
                "NAME ↑1".to_string(),
                "TIMESTAMP ↓2".to_string()
            ]
        )
    }
}
