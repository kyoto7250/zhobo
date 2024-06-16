<div align="center">

![zhobo](./resources/logo.png)

zhobo is currently in alpha

A cross-platform TUI database management tool written in Rust

[![github workflow status](https://img.shields.io/github/workflow/status/kyoto7250/zhobo/CI/main)](https://github.com/kyoto7250/zhobo/actions) [![crates](https://img.shields.io/crates/v/zhobo.svg?logo=rust)](https://crates.io/crates/zhobo)

![zhobo](./resources/zhobo.gif)

</div>

## Features

- Cross-platform support (macOS, Windows, Linux)
- Multiple Database support (MySQL, PostgreSQL, SQLite)
- Intuitive keyboard only control

## TODOs

- [ ] SQL editor
- [ ] Custom key bindings
- [ ] Custom theme settings
- [ ] Support the other databases

## What does "zhobo" come from?

zhobo means a Japanese game played on goban, a go board. The appearance of goban looks like table structure. And I live in Kyoto, Japan. In Kyoto city, streets are laid out on a grid (We call it “goban no me no youna (碁盤の目のような)”). They are why I named this project "zhobo".

## Installation

### With Homebrew (Linux, macOS)

If you’re using Homebrew or Linuxbrew, install the zhobo formula:

```
brew install kyoto7250/tap/zhobo
```

### On Windows

If you're a Windows Scoop user, then you can install zhobo from the [official bucket](https://github.com/ScoopInstaller/Main/blob/master/bucket/zhobo.json):

```
scoop install zhobo
```
### On NixOS

If you're a Nix user, you can install [zhobo](https://github.com/NixOS/nixpkgs/blob/master/pkgs/development/tools/database/zhobo/default.nix) from nixpkgs:

```
$ nix-env --install zhobo
```

### On Archlinux

If you're an Archlinux user, you can install [zhobo](https://aur.archlinux.org/packages/zhobo-bin) from AUR:

```
paru -S zhobo-bin
```

### On NetBSD

If you're a NetBSD user, then you can install zhobo from [pkgsrc](https://pkgsrc.se/databases/zhobo):

```
pkgin install zhobo
```

### With Cargo (Linux, macOS, Windows)

If you already have a Rust environment set up, you can use the `cargo install` command:

```
cargo install --version 0.1.0-alpha.5 zhobo
```

### From binaries (Linux, macOS, Windows)

- Download the [latest release binary](https://github.com/kyoto7250/zhobo/releases) for your system
- Set the `PATH` environment variable

## Usage

```
$ zhobo
```

```
$ zhobo -h
USAGE:
    zhobo [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config-path <config-path>    Set the config file
```

If you want to add connections, you need to edit your config file. For more information, please see [Configuration](#Configuration).

## Keymap

| Key | Description |
| ---- | ---- |
| <kbd>h</kbd>, <kbd>j</kbd>, <kbd>k</kbd>, <kbd>l</kbd> | Scroll left/down/up/right |
| <kbd>Ctrl</kbd> + <kbd>u</kbd>, <kbd>Ctrl</kbd> + <kbd>d</kbd> | Scroll up/down multiple lines |
| <kbd>g</kbd> , <kbd>G</kbd> | Scroll to top/bottom |
| <kbd>H</kbd>, <kbd>J</kbd>, <kbd>K</kbd>, <kbd>L</kbd> | Extend selection by one cell left/down/up/right |
| <kbd>y</kbd> | Copy a cell value |
| <kbd>←</kbd>, <kbd>→</kbd> | Move focus to left/right |
| <kbd>c</kbd> | Move focus to connections |
| <kbd>/</kbd> | Filter |
| <kbd>?</kbd> | Help |
| <kbd>1</kbd>, <kbd>2</kbd>, <kbd>3</kbd>, <kbd>4</kbd>, <kbd>5</kbd> | Switch to records/columns/constraints/foreign keys/indexes tab |
| <kbd>Esc</kbd> | Hide pop up |

## Configuration

The location of the file depends on your OS:

- macOS: `$HOME/.config/zhobo/config.toml`
- Linux: `$HOME/.config/zhobo/config.toml`
- Windows: `%APPDATA%/zhobo/config.toml`

The following is a sample config.toml file:

```toml
[[conn]]
type = "mysql"
user = "root"
host = "localhost"
port = 3306

[[conn]]
type = "mysql"
user = "root"
host = "localhost"
port = 3306
password = "password"
database = "foo"
name = "mysql Foo DB"

[[conn]]
type = "postgres"
user = "root"
host = "localhost"
port = 5432
database = "bar"
name = "postgres Bar DB"

[[conn]]
type = "sqlite"
path = "/path/to/baz.db"
```

## Contribution

Contributions, issues and pull requests are welcome!
