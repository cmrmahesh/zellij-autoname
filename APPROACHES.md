# Approaches to Auto-Renaming Zellij Tabs

This project offers two approaches. Choose based on your priorities.

## Option 1: WASM Plugin

A Zellij plugin (Rust/WASM) that runs in the background and renames tabs automatically.

**How it works:** Subscribes to Zellij's `TabUpdate` and `PaneUpdate` events. When a pane's title changes (reflecting a new directory or running command), the plugin renames the tab.

**Install:**
```kdl
// In ~/.config/zellij/config.kdl
load_plugins {
    "file:~/.config/zellij/plugins/zellij_autoname.wasm"
}
```

**Pros:**
- Shell-agnostic — works with zsh, bash, fish, or any shell
- No changes to shell config
- Works for all panes, including command panes and editor panes
- Single install point

**Cons:**
- Slight delay (~200-500ms) between directory change and tab rename. This is inherent to Zellij's plugin event system — `PaneUpdate` events are batched and don't fire instantly when the terminal title changes.
- Requires granting plugin permissions on first load

## Option 2: Shell Hooks (zsh)

Shell functions that call `zellij action rename-tab` directly from zsh hooks.

**How it works:** Uses zsh's `chpwd`, `precmd`, and `preexec` hooks to rename the tab immediately when you change directories or run a command.

**Install:**
```zsh
# In ~/.zshrc
source /path/to/zellij-autoname/shell/zellij-autoname.zsh
```

**Pros:**
- Instant — tab renames the moment you press Enter
- Zero latency, no event loop delay
- Simple, easy to customize

**Cons:**
- zsh-only (would need separate scripts for bash/fish)
- Requires modifying shell config
- Only works for the shell pane — won't rename for command panes or plugins

## Why is the shell approach faster?

The plugin relies on Zellij's event pipeline:

```
User runs `cd foo`
  → Shell updates terminal title (escape sequence)
  → Terminal emulator processes it
  → Zellij detects pane title change
  → Zellij batches and fires PaneUpdate event
  → Plugin receives event, calls rename_tab
  → Tab name updates in UI
```

The shell hook bypasses all of that:

```
User runs `cd foo`
  → zsh chpwd hook fires immediately
  → `zellij action rename-tab foo` runs
  → Tab name updates in UI
```

The shell hook runs *before* the terminal even processes the title change, so it's always faster.

## Option 3: Both

Use both approaches together. The shell hooks handle instant renames for your shell, while the plugin catches everything else (command panes, editor panes, other shells). They won't conflict — the plugin respects names that aren't default `Tab #N` names.
