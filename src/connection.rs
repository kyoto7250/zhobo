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
pub struct ReadConnection {
    r#type: DatabaseType,
    name: Option<String>,
    user: Option<String>,
    host: Option<String>,
    port: Option<u64>,
    path: Option<PathBuf>,
    password: Option<String>,
    unix_domain_socket: Option<PathBuf>,
    pub database: Option<String>,
    #[serde(default = "default_limit_size")]
    pub limit_size: usize,
    #[serde(default = "default_timeout_second")]
    pub timeout_second: u64,
}

#[derive(Debug, Clone)]
pub enum Connection {
    MySql(MySqlConnection),
    Postgres(PostgresConnection),
    Sqlite(SqliteConnection),
}

impl Connection {
    pub fn from(read_connection: ReadConnection) -> Self {
        match read_connection.r#type {
            DatabaseType::MySql => Connection::MySql(MySqlConnection {
                name: read_connection.name,
                user: read_connection
                    .user
                    .expect("user must be specified for MySQL"),
                password: read_connection.password,
                host: read_connection
                    .host
                    .expect("host must be specified for MySQL"),
                port: read_connection
                    .port
                    .expect("port must be specified for MySQL"),
                database: read_connection.database,
                unix_domain_socket: read_connection.unix_domain_socket,
                limit_size: read_connection.limit_size,
                timeout_second: read_connection.timeout_second,
            }),
            DatabaseType::Postgres => Connection::Postgres(PostgresConnection {
                name: read_connection.name,
                user: read_connection
                    .user
                    .expect("user must be specified for Postgres"),
                password: read_connection.password,
                host: read_connection
                    .host
                    .expect("host must be specified for Postgres"),
                port: read_connection
                    .port
                    .expect("port must be specified for Postgres"),
                database: read_connection.database,
                unix_domain_socket: read_connection.unix_domain_socket,
                limit_size: read_connection.limit_size,
                timeout_second: read_connection.timeout_second,
            }),
            DatabaseType::Sqlite => Connection::Sqlite(SqliteConnection {
                name: read_connection.name,
                path: read_connection
                    .path
                    .expect("path must be specified for Sqlite"),
                limit_size: read_connection.limit_size,
                timeout_second: read_connection.timeout_second,
            }),
        }
    }

    pub fn get_database(&self) -> Option<String> {
        match self {
            Connection::MySql(conn) => conn.database.clone(),
            Connection::Postgres(conn) => conn.database.clone(),
            Connection::Sqlite(conn) => conn.path.to_str().map(|s| s.to_string()),
        }
    }

    pub fn database_url(&self) -> anyhow::Result<String> {
        match self {
            Connection::MySql(conn) => conn.database_url(),
            Connection::Postgres(conn) => conn.database_url(),
            Connection::Sqlite(conn) => conn.database_url(),
        }
    }

    pub fn database_url_with_name(&self) -> anyhow::Result<String> {
        fn add_name_to_url(
            url: anyhow::Result<String>,
            name: Option<&String>,
        ) -> anyhow::Result<String> {
            match url {
                Ok(url) => Ok(match name {
                    Some(name) => {
                        format!("[{name}] {database_url}", name = name, database_url = url)
                    }
                    None => url,
                }),
                Err(e) => Err(anyhow::anyhow!(e).context(
                    "Failed to database_url_with_name in Connection::database_url_with_name",
                )),
            }
        }

        match self {
            Connection::MySql(conn) => {
                add_name_to_url(conn.database_url_with_masked_password(), conn.name.as_ref())
            }
            Connection::Postgres(conn) => {
                add_name_to_url(conn.database_url_with_masked_password(), conn.name.as_ref())
            }
            Connection::Sqlite(conn) => add_name_to_url(conn.database_url(), conn.name.as_ref()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MySqlConnection {
    name: Option<String>,
    user: String,
    password: Option<String>,
    host: String,
    port: u64,
    pub database: Option<String>,
    unix_domain_socket: Option<PathBuf>,
    pub limit_size: usize,
    pub timeout_second: u64,
}

impl MySqlConnection {
    pub fn database_url(&self) -> anyhow::Result<String> {
        let password = self
            .password
            .as_ref()
            .map_or(String::new(), |p| p.to_string());
        self.build_database_url(password)
    }

    pub fn database_url_with_masked_password(&self) -> anyhow::Result<String> {
        let password = self
            .password
            .as_ref()
            .map_or(String::new(), |p| p.to_string());

        let masked_password = "*".repeat(password.len());
        self.build_database_url(masked_password)
    }

    fn build_database_url(&self, password: String) -> anyhow::Result<String> {
        match self.database.as_ref() {
            Some(database) => Ok(format!(
                "mysql://{user}:{password}@{host}:{port}/{database}{unix_domain_socket}",
                user = self.user,
                password = password,
                host = self.host,
                port = self.port,
                database = database,
                unix_domain_socket = self.get_and_validate_unix_domain_socket()
            )),
            None => Ok(format!(
                "mysql://{user}:{password}@{host}:{port}{unix_domain_socket}",
                user = self.user,
                password = password,
                host = self.host,
                port = self.port,
                unix_domain_socket = self.get_and_validate_unix_domain_socket()
            )),
        }
    }

    fn get_and_validate_unix_domain_socket(&self) -> String {
        valid_unix_domain_socket(self.unix_domain_socket.clone())
            .map_or(String::new(), |uds| format!("?socket={}", uds))
    }
}

#[derive(Debug, Clone)]
pub struct PostgresConnection {
    name: Option<String>,
    user: String,
    password: Option<String>,
    host: String,
    port: u64,
    pub database: Option<String>,
    unix_domain_socket: Option<PathBuf>,
    pub limit_size: usize,
    pub timeout_second: u64,
}

impl PostgresConnection {
    pub fn database_url(&self) -> anyhow::Result<String> {
        let password = self
            .password
            .as_ref()
            .map_or(String::new(), |p| p.to_string());
        self.build_database_url(password)
    }

    pub fn database_url_with_masked_password(&self) -> anyhow::Result<String> {
        let password = self
            .password
            .as_ref()
            .map_or(String::new(), |p| p.to_string());

        let masked_password = "*".repeat(password.len());
        self.build_database_url(masked_password)
    }

    fn build_database_url(&self, password: String) -> anyhow::Result<String> {
        if let Some(unix_domain_socket) = self.get_and_validate_unix_domain_socket() {
            match self.database.as_ref() {
                Some(database) => Ok(format!(
                    "postgres://?dbname={database}&host={unix_domain_socket}&user={user}&password={password}",
                    database = database,
                    unix_domain_socket = unix_domain_socket,
                    user = self.user,
                    password = password,
                )),
                None => Ok(format!(
                    "postgres://?host={unix_domain_socket}&user={user}&password={password}",
                    unix_domain_socket = unix_domain_socket,
                    user = self.user,
                    password = password,
                )),
            }
        } else {
            match self.database.as_ref() {
                Some(database) => Ok(format!(
                    "postgres://{user}:{password}@{host}:{port}/{database}",
                    user = self.user,
                    password = password,
                    host = self.host,
                    port = self.port,
                    database = database,
                )),
                None => Ok(format!(
                    "postgres://{user}:{password}@{host}:{port}",
                    user = self.user,
                    password = password,
                    host = self.host,
                    port = self.port,
                )),
            }
        }
    }

    fn get_and_validate_unix_domain_socket(&self) -> Option<String> {
        valid_unix_domain_socket(self.unix_domain_socket.clone())
    }
}

#[derive(Debug, Clone)]
pub struct SqliteConnection {
    name: Option<String>,
    path: PathBuf,
    pub limit_size: usize,
    pub timeout_second: u64,
}

impl SqliteConnection {
    fn database_url(&self) -> anyhow::Result<String> {
        let path = expand_path(&self.path).ok_or_else(|| {
            anyhow::anyhow!("cannot expand file path in SqliteConnection:: build_database_url")
        })?;

        Ok(format!("sqlite://{path}", path = path.to_str().unwrap()))
    }
}

fn valid_unix_domain_socket(unix_domain_socket: Option<PathBuf>) -> Option<String> {
    if cfg!(windows) {
        // NOTE:
        // windows also supports UDS, but `rust` does not support UDS in windows now.
        // https://github.com/rust-lang/rust/issues/56533
        return None;
    }
    unix_domain_socket.as_ref().and_then(|uds| {
        let path = expand_path(uds)?;
        let path_str = path.to_str()?;
        if path_str.is_empty() {
            return None;
        }
        Some(path_str.to_owned())
    })
}

impl Default for Connection {
    fn default() -> Self {
        Connection::MySql(MySqlConnection {
            name: None,
            user: "root".to_string(),
            host: "localhost".to_string(),
            port: 3306,
            password: None,
            database: None,
            unix_domain_socket: None,
            limit_size: default_limit_size(),
            timeout_second: default_timeout_second(),
        })
    }
}

pub fn default_limit_size() -> usize {
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
    use super::*;
    use std::env;

    #[test]
    fn test_default() {
        let conn = Connection::default();
        match conn {
            Connection::MySql(mysql) => {
                assert_eq!(mysql.user, "root");
                assert_eq!(mysql.host, "localhost");
                assert_eq!(mysql.port, 3306);
                assert_eq!(mysql.limit_size, 200);
                assert_eq!(mysql.timeout_second, 5);
            }
            _ => panic!("Default should be MySql"),
        }
    }
    mod mysql_connection_tests {
        use super::*;

        #[test]
        fn database_url() {
            let mysql_conn = Connection::MySql(MySqlConnection {
                name: None,
                user: "root".to_owned(),
                host: "localhost".to_owned(),
                port: 3306,
                password: Some("password".to_owned()),
                database: Some("city".to_owned()),
                unix_domain_socket: None,
                limit_size: 200,
                timeout_second: 5,
            });

            let mysql_result = mysql_conn.database_url().unwrap();
            assert_eq!(
                mysql_result,
                "mysql://root:password@localhost:3306/city".to_owned()
            );
        }

        #[test]
        fn database_url_with_name() {
            let mysql_conn = Connection::MySql(MySqlConnection {
                name: Some("my_mysql_connection".to_owned()),
                user: "root".to_owned(),
                host: "localhost".to_owned(),
                port: 3306,
                password: Some("password".to_owned()),
                database: Some("city".to_owned()),
                unix_domain_socket: None,
                limit_size: 200,
                timeout_second: 5,
            });

            let mysql_result = mysql_conn.database_url_with_name().unwrap();
            assert_eq!(
                mysql_result,
                "[my_mysql_connection] mysql://root:********@localhost:3306/city".to_owned()
            );
        }

        #[test]
        #[cfg(unix)]
        fn database_url_in_unix_includes_socket() {
            let mysql_conn = Connection::MySql(MySqlConnection {
                name: None,
                user: "root".to_owned(),
                host: "localhost".to_owned(),
                port: 3306,
                password: Some("password".to_owned()),
                database: Some("city".to_owned()),
                unix_domain_socket: Some(Path::new("/tmp/mysql.sock").to_path_buf()),
                limit_size: 200,
                timeout_second: 5,
            });

            assert_eq!(
                mysql_conn.database_url().unwrap(),
                "mysql://root:password@localhost:3306/city?socket=/tmp/mysql.sock".to_owned()
            );
        }

        #[test]
        #[cfg(windows)]
        fn database_url_in_windows_ignores_socket() {
            let mysql_conn = Connection::MySql(MySqlConnection {
                name: None,
                user: "root".to_owned(),
                host: "localhost".to_owned(),
                port: 3306,
                password: Some("password".to_owned()),
                database: Some("city".to_owned()),
                unix_domain_socket: "/tmp/mysql.sock".to_owned(),
                limit_size: 200,
                timeout_second: 5,
            });

            assert_eq!(
                mysql_conn.database_url().unwrap(),
                "mysql://root:password@localhost:3306/city".to_owned()
            );
        }
    }

    mod postgres_connection_tests {
        use super::*;

        #[test]
        fn database_url() {
            let postgres_conn = Connection::Postgres(PostgresConnection {
                name: None,
                user: "root".to_owned(),
                host: "localhost".to_owned(),
                port: 3306,
                password: Some("password".to_owned()),
                database: Some("city".to_owned()),
                unix_domain_socket: None,
                limit_size: 200,
                timeout_second: 5,
            });

            let postgres_result = postgres_conn.database_url().unwrap();
            assert_eq!(
                postgres_result,
                "postgres://root:password@localhost:3306/city".to_owned()
            );
        }

        #[test]
        fn database_url_with_name() {
            let postgres_conn = Connection::Postgres(PostgresConnection {
                name: Some("my_postgres_connection".to_owned()),
                user: "root".to_owned(),
                host: "localhost".to_owned(),
                port: 3306,
                password: Some("password".to_owned()),
                database: Some("city".to_owned()),
                unix_domain_socket: None,
                limit_size: 200,
                timeout_second: 5,
            });

            let postgres_result = postgres_conn.database_url_with_name().unwrap();
            assert_eq!(
                postgres_result,
                "[my_postgres_connection] postgres://root:********@localhost:3306/city".to_owned()
            );
        }

        #[test]
        #[cfg(unix)]
        fn database_url_in_unix_includes_socket() {
            let postgres_conn = Connection::Postgres(PostgresConnection {
                name: None,
                user: "root".to_owned(),
                host: "localhost".to_owned(),
                port: 3306,
                password: Some("password".to_owned()),
                database: Some("city".to_owned()),
                unix_domain_socket: Some(Path::new("/tmp").to_path_buf()),
                limit_size: 200,
                timeout_second: 5,
            });

            assert_eq!(
                postgres_conn.database_url().unwrap(),
                "postgres://?dbname=city&host=/tmp&user=root&password=password".to_owned()
            );
        }

        #[test]
        #[cfg(windows)]
        fn database_url_in_windows_ignores_socket() {
            let postgres_conn = Connection::Postgres(PostgresConnection {
                name: None,
                user: "root".to_owned(),
                host: "localhost".to_owned(),
                port: 3306,
                password: Some("password".to_owned()),
                database: Some("city".to_owned()),
                unix_domain_socket: Some("/tmp".to_owned()),
                limit_size: 200,
                timeout_second: 5,
            });

            assert_eq!(
                postgres_conn.database_url().unwrap(),
                "postgres://root:password@localhost:3306/city".to_owned()
            );
        }
    }

    mod sqlite_connection_tests {
        use super::*;

        #[test]
        fn database_url() {
            let sqlite_conn = Connection::Sqlite(SqliteConnection {
                name: None,
                path: PathBuf::from("/home/user/sqlite3.db"),
                limit_size: 200,
                timeout_second: 5,
            });

            let sqlite_result = sqlite_conn.database_url().unwrap();
            assert_eq!(sqlite_result, "sqlite:///home/user/sqlite3.db".to_owned());
        }

        #[test]
        fn database_url_with_name() {
            let sqlite_conn = Connection::Sqlite(SqliteConnection {
                name: Some("my_sqlite_connection".to_owned()),
                path: PathBuf::from("/home/user/sqlite3.db"),
                limit_size: 200,
                timeout_second: 5,
            });

            let sqlite_result = sqlite_conn.database_url_with_name().unwrap();
            assert_eq!(
                sqlite_result,
                "[my_sqlite_connection] sqlite:///home/user/sqlite3.db".to_owned()
            );
        }

        #[test]
        #[cfg(windows)]
        fn database_url_in_windows() {
            let sqlite_conn = Connection::Sqlite(SqliteConnection {
                name: None,
                path: PathBuf::from("/home/user/sqlite3.db"),
                limit_size: 200,
                timeout_second: 5,
            });

            let sqlite_result = sqlite_conn.database_url().unwrap();
            assert_eq!(
                sqlite_result,
                "sqlite://\\home\\user\\sqlite3.db".to_owned()
            );
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_expand_path() {
        let home = env::var("HOME").unwrap();
        let test_env = "baz";
        env::set_var("TEST", test_env);

        assert_eq!(
            expand_path(Path::new("$HOME/foo")),
            Some(PathBuf::from(&home).join("foo"))
        );

        assert_eq!(
            expand_path(Path::new("$HOME/foo/$TEST/bar")),
            Some(PathBuf::from(&home).join("foo").join(test_env).join("bar"))
        );

        assert_eq!(
            expand_path(Path::new("~/foo")),
            Some(PathBuf::from(&home).join("foo"))
        );

        assert_eq!(
            expand_path(Path::new("~/foo/~/bar")),
            Some(PathBuf::from(&home).join("foo").join("~").join("bar"))
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_expand_path() {
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
