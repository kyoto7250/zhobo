use crate::connection::{Connection, ReadConnection};
use crate::key_bind::KeyBind;
use crate::log::LogLevel;
use crate::Key;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use structopt::StructOpt;

#[cfg(test)]
use serde::Serialize;

#[derive(StructOpt, Debug)]
pub struct CliConfig {
    /// Set the config file
    #[structopt(long, short, global = true)]
    config_path: Option<std::path::PathBuf>,

    /// Set the key bind file
    #[structopt(long, short, global = true)]
    key_bind_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReadConfig {
    pub conn: Vec<ReadConnection>,
    #[serde(default)]
    pub log_level: LogLevel,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub conn: Vec<Connection>,
    pub key_config: KeyConfig,
    pub log_level: LogLevel,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            conn: vec![Connection::default()],
            key_config: KeyConfig::default(),
            log_level: LogLevel::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(Serialize, PartialEq))]
pub struct KeyConfig {
    pub scroll_up: Key,
    pub scroll_down: Key,
    pub scroll_right: Key,
    pub scroll_left: Key,
    pub sort_by_column: Key,
    pub move_up: Key,
    pub move_down: Key,
    pub copy: Key,
    pub enter: Key,
    pub exit: Key,
    pub quit: Key,
    pub exit_popup: Key,
    pub focus_right: Key,
    pub focus_left: Key,
    pub focus_above: Key,
    pub focus_connections: Key,
    pub open_help: Key,
    pub filter: Key,
    pub scroll_down_multiple_lines: Key,
    pub scroll_up_multiple_lines: Key,
    pub scroll_to_top: Key,
    pub scroll_to_bottom: Key,
    pub move_to_head_of_line: Key,
    pub move_to_tail_of_line: Key,
    pub extend_selection_by_one_cell_left: Key,
    pub extend_selection_by_one_cell_right: Key,
    pub extend_selection_by_one_cell_up: Key,
    pub extend_selection_by_one_cell_down: Key,
    pub extend_selection_by_horizontal_line: Key,
    pub tab_records: Key,
    pub tab_columns: Key,
    pub tab_constraints: Key,
    pub tab_definition: Key,
    pub tab_foreign_keys: Key,
    pub tab_indexes: Key,
    pub tab_sql_editor: Key,
    pub tab_properties: Key,
    pub extend_or_shorten_widget_width_to_right: Key,
    pub extend_or_shorten_widget_width_to_left: Key,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            scroll_up: Key::Char('k'),
            scroll_down: Key::Char('j'),
            scroll_right: Key::Char('l'),
            scroll_left: Key::Char('h'),
            sort_by_column: Key::Char('s'),
            move_up: Key::Up,
            move_down: Key::Down,
            copy: Key::Char('y'),
            enter: Key::Enter,
            exit: Key::Ctrl('c'),
            quit: Key::Char('q'),
            exit_popup: Key::Esc,
            focus_right: Key::Right,
            focus_left: Key::Left,
            focus_above: Key::Up,
            focus_connections: Key::Char('c'),
            open_help: Key::Char('?'),
            filter: Key::Char('/'),
            scroll_down_multiple_lines: Key::Ctrl('d'),
            scroll_up_multiple_lines: Key::Ctrl('u'),
            scroll_to_top: Key::Char('g'),
            scroll_to_bottom: Key::Char('G'),
            move_to_head_of_line: Key::Char('^'),
            move_to_tail_of_line: Key::Char('$'),
            extend_selection_by_one_cell_left: Key::Char('H'),
            extend_selection_by_one_cell_right: Key::Char('L'),
            extend_selection_by_one_cell_down: Key::Char('J'),
            extend_selection_by_horizontal_line: Key::Char('V'),
            extend_selection_by_one_cell_up: Key::Char('K'),
            tab_records: Key::Char('1'),
            tab_properties: Key::Char('2'),
            tab_sql_editor: Key::Char('3'),
            tab_columns: Key::Char('4'),
            tab_constraints: Key::Char('5'),
            tab_foreign_keys: Key::Char('6'),
            tab_indexes: Key::Char('7'),
            tab_definition: Key::Char('8'),
            extend_or_shorten_widget_width_to_right: Key::Char('>'),
            extend_or_shorten_widget_width_to_left: Key::Char('<'),
        }
    }
}

impl Config {
    pub fn new(config: &CliConfig) -> anyhow::Result<Self> {
        let config_path = if let Some(config_path) = &config.config_path {
            config_path.clone()
        } else {
            get_app_config_path()?.join("config.toml")
        };

        let key_bind_path = if let Some(key_bind_path) = &config.key_bind_path {
            key_bind_path.clone()
        } else {
            get_app_config_path()?.join("key_bind.ron")
        };

        if let Ok(file) = File::open(config_path) {
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents)?;
            let config: Result<ReadConfig, toml::de::Error> = toml::from_str(&contents);
            match config {
                Ok(config) => return Ok(Config::build(config, key_bind_path)),
                Err(e) => panic!("fail to parse connection config file: {}", e),
            }
        }

        Ok(Config::default())
    }

    fn build(read_config: ReadConfig, key_bind_path: PathBuf) -> Self {
        let key_bind = KeyBind::load(key_bind_path).unwrap();
        Config {
            conn: read_config
                .conn
                .into_iter()
                .map(|c| Connection::from(c))
                .collect::<Vec<Connection>>(),
            log_level: read_config.log_level,
            key_config: KeyConfig::from(key_bind),
        }
    }
}

pub fn get_app_config_path() -> anyhow::Result<std::path::PathBuf> {
    let mut path = if cfg!(target_os = "macos") {
        dirs_next::home_dir().map(|h| h.join(".config"))
    } else {
        dirs_next::config_dir()
    }
    .ok_or_else(|| anyhow::anyhow!("failed to find os config dir."))?;

    path.push("zhobo");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::{CliConfig, Config, KeyConfig};
    use serde_json::Value;

    #[test]
    fn test_load_config() {
        let cli_config = CliConfig {
            config_path: Some(Path::new("examples/config.toml").to_path_buf()),
            key_bind_path: Some(Path::new("examples/key_bind.ron").to_path_buf()),
        };

        assert_eq!(Config::new(&cli_config).is_ok(), true);
    }

    #[test]
    fn test_overlappted_key() {
        let value: Value =
            serde_json::from_str(&serde_json::to_string(&KeyConfig::default()).unwrap()).unwrap();
        if let Value::Object(map) = value {
            let mut values: Vec<String> = map
                .values()
                .map(|v| match v {
                    Value::Object(map) => Some(format!("{:?}", map)),
                    _ => None,
                })
                .flatten()
                .collect();
            values.sort();
            let before_values = values.clone();
            values.dedup();
            pretty_assertions::assert_eq!(before_values, values);
        }
    }
}
