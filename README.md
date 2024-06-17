# zhobo

![](https://github.com/kyoto7250/zhobo/workflows/CI/badge.svg)
![](https://github.com/kyoto7250/zhobo/workflows/Release/badge.svg)


`zhobo` is the rebaked [gobang project](https://github.com/TaKO8Ki/gobang).

## Features
- Cross-platform support (macOS, Windows, Linux)
- Multiple Database support (MySQL, PostgreSQL, SQLite)
- Intuitive keyboard only control

## Additional Features
- [x] custom keymap.
- [x] support unix domain.
- [x] sort based on specific columns.

## installation

## homebrew

## cargo
```bash
cargo install zhobo
```

## default keymap

| Key | Description |
| ---- | ---- |
| <kbd>h</kbd>, <kbd>j</kbd>, <kbd>k</kbd>, <kbd>l</kbd> | Scroll left/down/up/right |
| <kbd>Ctrl</kbd> + <kbd>u</kbd>, <kbd>Ctrl</kbd> + <kbd>d</kbd> | Scroll up/down multiple lines |
| <kbd>g</kbd> , <kbd>G</kbd> | Scroll to top/bottom |
| <kbd>^</kbd>, <kbd>$</kbd> | Move to head/tail of line |
| <kbd>s</kbd> | Sort by selected column |
| <kbd>H</kbd>, <kbd>J</kbd>, <kbd>K</kbd>, <kbd>L</kbd> | Extend selection by one cell left/down/up/right |
| <kbd>V</kbd> | Extend selection by horizontal line |
| <kbd>y</kbd> | Copy a cell value |
| <kbd>←</kbd>, <kbd>→</kbd> | Move focus to left/right |
| <kbd>c</kbd> | Move focus to connections |
| <kbd>/</kbd> | Filter |
| <kbd>?</kbd> | Help |
| <kbd>1</kbd>, <kbd>2</kbd>, <kbd>3</kbd>, <kbd>4</kbd>, <kbd>5</kbd> | Switch to records/columns/constraints/foreign keys/indexes tab |
| <kbd>Esc</kbd> | Hide pop up |


## configuration

### connection

The location of the file depends on your OS:

- macOS: `$HOME/.config/zhobo/config.toml`
- Linux: `$HOME/.config/zhobo/config.toml`
- Windows: `%APPDATA%/zhobo/config.toml`

Sample config.toml file is `examples/config.toml`:

### custom keymap

The location of the file depends on your OS:

- macOS: `$HOME/.config/zhobo/key_bind.ron`
- Linux: `$HOME/.config/zhobo/key_bind.ron`
- Windows: `%APPDATA%/zhobo/key_bind.ron`

Sample config.toml file is `examples/key_bind.ron`:

## contribution

Contributions are welcome.
If you are developing a new feature, we recommend creating an issue first.

## acknowledge

Most of the code in this project was ported from [gobang](https://github.com/TaKO8Ki/gobang), and we would like to express our deepest gratitude to the original author, [@Tako8ki](https://github.com/TaKO8Ki).