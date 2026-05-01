# zellij-autoname

Auto-rename Zellij tabs based on the focused pane's running command or working directory.

No more `Tab #1`, `Tab #2` — see `vim`, `cargo build`, or `my-project` at a glance.

New tabs in Zellij just come up as `Tab #1`, `Tab #2`, etc. — which tells you nothing when you have several open. This plugin automatically renames them to show either the currently running command or the basename of the working directory, so you always know what's where.

## Design Principles

- **Priority: running command > pwd basename.** If a command is actively running (e.g. `vim`, `cargo test`), show that. When idle, fall back to the directory name.
- **Short names:** use just the basename of pwd (e.g. `zellij` not `/Users/you/workspace/zellij`), and truncate long command names.
- **Don't rename user-named tabs:** if someone manually names a tab, respect that.
- **Language:** Zellij plugins are written in Rust (compiled to WASM). The `zellij-tile` crate provides the plugin API.

## Features

- **Auto-rename**: tabs show the running command (e.g. `vim`, `cargo`) or the current directory basename
- **Respects manual names**: if you rename a tab yourself, the plugin leaves it alone
- **RenameTab-aware**: pauses auto-naming while you're typing a tab name
- **Background plugin**: invisible, non-interactive, zero UI overhead
- **Configurable**: set max tab name length

## Install

### From release

Download the `.wasm` file from [Releases](https://github.com/YOUR_USER/zellij-autoname/releases) and place it somewhere accessible (e.g. `~/.config/zellij/plugins/`).

### Build from source

```bash
make setup        # one-time: rustup target add wasm32-wasip1
make              # build release wasm
make install      # copy to ~/.config/zellij/plugins/
```

Equivalent without Make:

```bash
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/zellij_autoname.wasm ~/.config/zellij/plugins/
```

Other targets: `make debug`, `make check`, `make fmt`, `make uninstall`, `make clean`.

## Configuration

Add the plugin to your Zellij layout or config. It runs as a hidden background plugin.

### In a layout file (`~/.config/zellij/layouts/default.kdl`)

```kdl
layout {
    pane

    pane {
        plugin location="file:~/.config/zellij/plugins/zellij_autoname.wasm" {
            max_length "20"
        }
    }
}
```

### As a keybinding (load on demand)

```kdl
keybinds {
    shared {
        bind "Ctrl Shift n" {
            LaunchPlugin "file:~/.config/zellij/plugins/zellij_autoname.wasm" {
                floating true
                max_length "20"
            }
        }
    }
}
```

### Auto-start in config (`~/.config/zellij/config.kdl`)

```kdl
plugins {
    autoname location="file:~/.config/zellij/plugins/zellij_autoname.wasm" {
        max_length "20"
    }
}

// Then reference it in your layout:
// pane { plugin location="autoname" }
```

## Shell hooks (optional, zsh)

Wasm plugin only fires on Zellij events. For instant tab rename on `cd` / command exec, also source the zsh hook:

```bash
# in ~/.zshrc
source /path/to/zellij-autoname/shell/zellij-autoname.zsh
```

Renames tab to current dir basename on `cd`/precmd, and to running command on preexec (e.g. `vim`, `cargo`). Use either the wasm plugin, the shell hook, or both.

## Options

| Key          | Default | Description                    |
|--------------|---------|--------------------------------|
| `max_length` | `20`    | Max characters for a tab name  |

## How it works

1. Subscribes to `TabUpdate`, `PaneUpdate`, and `ModeUpdate` events
2. For each tab with a default name (`Tab #N`):
   - Finds the focused non-plugin pane
   - If a command is running → uses the command name (e.g. `vim`, `cargo`)
   - Otherwise → uses the directory basename from the pane title
3. Skips tabs the user has manually renamed
4. Pauses while the user is in RenameTab mode

## Requirements

- Zellij ≥ 0.40.0 (WASI plugin support)
- Grants `ChangeApplicationState` and `ReadApplicationState` permissions when prompted

## License

MIT
