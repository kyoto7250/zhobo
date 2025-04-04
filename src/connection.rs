use std::{
    fmt,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
enum DatabaseType {
    #[serde(rename = "mysql")]
    MySql,
    #[serde(rename = "postgres")]
    Postgres,
    #[serde(rename = "sqlite")]
    Sqlite,
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MySql => write!(f, "mysql"),
            Self::Postgres => write!(f, "postgres"),
            Self::Sqlite => write!(f, "sqlite"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Connection {
    r#type: DatabaseType,
    name: Option<String>,
    user: Option<String>,
    host: Option<String>,
    port: Option<u64>,
    path: Option<std::path::PathBuf>,
    password: Option<String>,
    unix_domain_socket: Option<std::path::PathBuf>,
    pub database: Option<String>,
    #[serde(default = "default_limit_size")]
    pub limit_size: usize,
    #[serde(default = "default_timeout_second")]
    pub timeout_second: u64,
}

impl Connection {
    pub fn database_url(&self) -> anyhow::Result<String> {
        let password = self
            .password
            .as_ref()
            .map_or(String::new(), |p| p.to_string());
        self.build_database_url(password)
    }

    fn masked_database_url(&self) -> anyhow::Result<String> {
        let password = self
            .password
            .as_ref()
            .map_or(String::new(), |p| p.to_string());

        let masked_password = "*".repeat(password.len());
        self.build_database_url(masked_password)
    }

    fn build_database_url(&self, password: String) -> anyhow::Result<String> {
        match self.r#type {
            DatabaseType::MySql => {
                let user = self.user.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "type mysql needs the user field in Connection::build_database_url"
                    )
                })?;
                let host = self.host.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "type mysql needs the host field in Connection::build_database_url"
                    )
                })?;
                let port = self.port.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "type mysql needs the port field in Connection::build_database_url"
                    )
                })?;
                let unix_domain_socket = self
                    .valid_unix_domain_socket()
                    .map_or(String::new(), |uds| format!("?socket={}", uds));

                match self.database.as_ref() {
                    Some(database) => Ok(format!(
                        "mysql://{user}:{password}@{host}:{port}/{database}{unix_domain_socket}",
                        user = user,
                        password = password,
                        host = host,
                        port = port,
                        database = database,
                        unix_domain_socket = unix_domain_socket
                    )),
                    None => Ok(format!(
                        "mysql://{user}:{password}@{host}:{port}{unix_domain_socket}",
                        user = user,
                        password = password,
                        host = host,
                        port = port,
                        unix_domain_socket = unix_domain_socket
                    )),
                }
            }
            DatabaseType::Postgres => {
                let user = self.user.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "type postgres needs the user field in Connection::build_database_url"
                    )
                })?;
                let host = self.host.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "type postgres needs the host field in Connection::build_database_url"
                    )
                })?;
                let port = self.port.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "type postgres needs the port field in Connection::build_database_url"
                    )
                })?;

                if let Some(unix_domain_socket) = self.valid_unix_domain_socket() {
                    match self.database.as_ref() {
                        Some(database) => Ok(format!(
                            "postgres://?dbname={database}&host={unix_domain_socket}&user={user}&password={password}",
                            database = database,
                            unix_domain_socket = unix_domain_socket,
                            user = user,
                            password = password,
                        )),
                        None => Ok(format!(
                            "postgres://?host={unix_domain_socket}&user={user}&password={password}",
                            unix_domain_socket = unix_domain_socket,
                            user = user,
                            password = password,
                        )),
                    }
                } else {
                    match self.database.as_ref() {
                        Some(database) => Ok(format!(
                            "postgres://{user}:{password}@{host}:{port}/{database}",
                            user = user,
                            password = password,
                            host = host,
                            port = port,
                            database = database,
                        )),
                        None => Ok(format!(
                            "postgres://{user}:{password}@{host}:{port}",
                            user = user,
                            password = password,
                            host = host,
                            port = port,
                        )),
                    }
                }
            }
            DatabaseType::Sqlite => {
                let path = self.path.as_ref().map_or(
                    Err(anyhow::anyhow!(
                        "type sqlite needs the path field in Connection::build_database_url"
                    )),
                    |path| {
                        expand_path(path).ok_or_else(|| {
                            anyhow::anyhow!(
                                "cannot expand file path in Connection::build_database_url"
                            )
                        })
                    },
                )?;

                Ok(format!("sqlite://{path}", path = path.to_str().unwrap()))
            }
        }
    }

    pub fn database_url_with_name(&self) -> anyhow::Result<String> {
        match self.masked_database_url() {
            Ok(url) => Ok(match &self.name {
                Some(name) => format!("[{name}] {database_url}", name = name, database_url = url),
                None => url,
            }),
            Err(e) => Err(anyhow::anyhow!(e)
                .context("Failed to masked_database_url in Connection::database_url_with_name")),
        }
    }

    pub fn is_mysql(&self) -> bool {
        matches!(self.r#type, DatabaseType::MySql)
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self.r#type, DatabaseType::Postgres)
    }

    fn valid_unix_domain_socket(&self) -> Option<String> {
        if cfg!(windows) {
            // NOTE:
            // windows also supports UDS, but `rust` does not support UDS in windows now.
            // https://github.com/rust-lang/rust/issues/56533
            return None;
        }
        return self.unix_domain_socket.as_ref().and_then(|uds| {
            let path = expand_path(uds)?;
            let path_str = path.to_str()?;
            if path_str.is_empty() {
                return None;
            }
            Some(path_str.to_owned())
        });
    }
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            r#type: DatabaseType::MySql,
            name: None,
            user: Some("root".to_string()),
            host: Some("localhost".to_string()),
            port: Some(3306),
            path: None,
            password: None,
            database: None,
            unix_domain_socket: None,
            limit_size: default_limit_size(),
            timeout_second: default_timeout_second(),
        }
    }
}

fn default_limit_size() -> usize {
    200
}

fn default_timeout_second() -> u64 {
    5
}

fn expand_path(path: &Path) -> Option<PathBuf> {
    let mut expanded_path = PathBuf::new();
    let mut path_iter = path.iter();
    if path.starts_with("~") {
        path_iter.next()?;
        expanded_path = expanded_path.join(dirs_next::home_dir()?);
    }
    for path in path_iter {
        let path = path.to_str()?;
        expanded_path = if cfg!(unix) && path.starts_with('$') {
            expanded_path.join(std::env::var(path.strip_prefix('$')?).unwrap_or_default())
        } else if cfg!(windows) && path.starts_with('%') && path.ends_with('%') {
            expanded_path
                .join(std::env::var(path.strip_prefix('%')?.strip_suffix('%')?).unwrap_or_default())
        } else {
            expanded_path.join(path)
        }
    }
    Some(expanded_path)
}

#[cfg(test)]
mod test {
    use super::{expand_path, Connection, DatabaseType, Path, PathBuf};
    use std::env;

    #[test]
    #[cfg(unix)]
    fn test_database_url() {
        let mysql_conn = Connection {
            r#type: DatabaseType::MySql,
            name: None,
            user: Some("root".to_owned()),
            host: Some("localhost".to_owned()),
            port: Some(3306),
            path: None,
            password: Some("password".to_owned()),
            database: Some("city".to_owned()),
            unix_domain_socket: None,
            limit_size: 200,
            timeout_second: 5,
        };

        let mysql_result = mysql_conn.database_url().unwrap();
        assert_eq!(
            mysql_result,
            "mysql://root:password@localhost:3306/city".to_owned()
        );

        let postgres_conn = Connection {
            r#type: DatabaseType::Postgres,
            name: None,
            user: Some("root".to_owned()),
            host: Some("localhost".to_owned()),
            port: Some(3306),
            path: None,
            password: Some("password".to_owned()),
            database: Some("city".to_owned()),
            unix_domain_socket: None,
            limit_size: 200,
            timeout_second: 5,
        };

        let postgres_result = postgres_conn.database_url().unwrap();
        assert_eq!(
            postgres_result,
            "postgres://root:password@localhost:3306/city".to_owned()
        );

        let sqlite_conn = Connection {
            r#type: DatabaseType::Sqlite,
            name: None,
            user: None,
            host: None,
            port: None,
            path: Some(PathBuf::from("/home/user/sqlite3.db")),
            password: None,
            database: None,
            unix_domain_socket: None,
            limit_size: 200,
            timeout_second: 5,
        };

        let sqlite_result = sqlite_conn.database_url().unwrap();
        assert_eq!(sqlite_result, "sqlite:///home/user/sqlite3.db".to_owned());
    }

    #[test]
    #[cfg(unix)]
    fn test_dataset_url_in_unix() {
        let mut mysql_conn = Connection {
            r#type: DatabaseType::MySql,
            name: None,
            user: Some("root".to_owned()),
            host: Some("localhost".to_owned()),
            port: Some(3306),
            path: None,
            password: Some("password".to_owned()),
            database: Some("city".to_owned()),
            unix_domain_socket: None,
            limit_size: 200,
            timeout_second: 5,
        };

        assert_eq!(
            mysql_conn.database_url().unwrap(),
            "mysql://root:password@localhost:3306/city".to_owned()
        );

        mysql_conn.unix_domain_socket = Some(Path::new("/tmp/mysql.sock").to_path_buf());
        assert_eq!(
            mysql_conn.database_url().unwrap(),
            "mysql://root:password@localhost:3306/city?socket=/tmp/mysql.sock".to_owned()
        );

        let mut postgres_conn = Connection {
            r#type: DatabaseType::Postgres,
            name: None,
            user: Some("root".to_owned()),
            host: Some("localhost".to_owned()),
            port: Some(3306),
            path: None,
            password: Some("password".to_owned()),
            database: Some("city".to_owned()),
            unix_domain_socket: None,
            limit_size: 200,
            timeout_second: 5,
        };

        assert_eq!(
            postgres_conn.database_url().unwrap(),
            "postgres://root:password@localhost:3306/city".to_owned()
        );
        postgres_conn.unix_domain_socket = Some(Path::new("/tmp").to_path_buf());
        assert_eq!(
            postgres_conn.database_url().unwrap(),
            "postgres://?dbname=city&host=/tmp&user=root&password=password".to_owned()
        );

        let sqlite_conn = Connection {
            r#type: DatabaseType::Sqlite,
            name: None,
            user: None,
            host: None,
            port: None,
            path: Some(PathBuf::from("/home/user/sqlite3.db")),
            password: None,
            database: None,
            unix_domain_socket: None,
            limit_size: 200,
            timeout_second: 5,
        };

        let sqlite_result = sqlite_conn.database_url().unwrap();
        assert_eq!(sqlite_result, "sqlite:///home/user/sqlite3.db".to_owned());
    }

    #[test]
    #[cfg(windows)]
    fn test_database_url_in_windows() {
        let mut mysql_conn = Connection {
            r#type: DatabaseType::MySql,
            name: None,
            user: Some("root".to_owned()),
            host: Some("localhost".to_owned()),
            port: Some(3306),
            path: None,
            password: Some("password".to_owned()),
            database: Some("city".to_owned()),
            unix_domain_socket: None,
            limit_size: 200,
            timeout_second: 5,
        };

        assert_eq!(
            mysql_conn.database_url().unwrap(),
            "mysql://root:password@localhost:3306/city".to_owned()
        );

        mysql_conn.unix_domain_socket = Some(Path::new("/tmp/mysql.sock").to_path_buf());
        assert_eq!(
            mysql_conn.database_url().unwrap(),
            "mysql://root:password@localhost:3306/city".to_owned()
        );

        let mut postgres_conn = Connection {
            r#type: DatabaseType::Postgres,
            name: None,
            user: Some("root".to_owned()),
            host: Some("localhost".to_owned()),
            port: Some(3306),
            path: None,
            password: Some("password".to_owned()),
            database: Some("city".to_owned()),
            unix_domain_socket: None,
            limit_size: 200,
            timeout_second: 5,
        };

        assert_eq!(
            postgres_conn.database_url().unwrap(),
            "postgres://root:password@localhost:3306/city".to_owned()
        );
        postgres_conn.unix_domain_socket = Some(Path::new("/tmp").to_path_buf());
        assert_eq!(
            postgres_conn.database_url().unwrap(),
            "postgres://root:password@localhost:3306/city".to_owned()
        );

        let sqlite_conn = Connection {
            r#type: DatabaseType::Sqlite,
            name: None,
            user: None,
            host: None,
            port: None,
            path: Some(PathBuf::from("/home/user/sqlite3.db")),
            password: None,
            database: None,
            unix_domain_socket: None,
            limit_size: 200,
            timeout_second: 5,
        };

        let sqlite_result = sqlite_conn.database_url().unwrap();
        assert_eq!(
            sqlite_result,
            "sqlite://\\home\\user\\sqlite3.db".to_owned()
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_expand_path() {
        let home = env::var("HOME").unwrap();
        let test_env = "baz";
        env::set_var("TEST", test_env);

        assert_eq!(
            expand_path(&Path::new("$HOME/foo")),
            Some(PathBuf::from(&home).join("foo"))
        );

        assert_eq!(
            expand_path(&Path::new("$HOME/foo/$TEST/bar")),
            Some(PathBuf::from(&home).join("foo").join(test_env).join("bar"))
        );

        assert_eq!(
            expand_path(&Path::new("~/foo")),
            Some(PathBuf::from(&home).join("foo"))
        );

        assert_eq!(
            expand_path(&Path::new("~/foo/~/bar")),
            Some(PathBuf::from(&home).join("foo").join("~").join("bar"))
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_expand_patha() {
        let home = std::env::var("HOMEPATH").unwrap();
        let test_env = "baz";
        env::set_var("TEST", test_env);

        assert_eq!(
            expand_path(&Path::new("%HOMEPATH%/foo")),
            Some(PathBuf::from(&home).join("foo"))
        );

        assert_eq!(
            expand_path(&Path::new("%HOMEPATH%/foo/%TEST%/bar")),
            Some(PathBuf::from(&home).join("foo").join(test_env).join("bar"))
        );

        assert_eq!(
            expand_path(&Path::new("~/foo")),
            Some(PathBuf::from(&dirs_next::home_dir().unwrap()).join("foo"))
        );

        assert_eq!(
            expand_path(&Path::new("~/foo/~/bar")),
            Some(
                PathBuf::from(&dirs_next::home_dir().unwrap())
                    .join("foo")
                    .join("~")
                    .join("bar")
            )
        );
    }
}
