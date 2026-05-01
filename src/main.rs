// zellij-autoname: keep tab names and pane names in sync with the focused pane's
// working directory or running command. The plugin reads each focused pane's
// title (set by the shell via OSC) and rewrites it into a short, useful label.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    tabs: Vec<TabInfo>,
    panes: HashMap<usize, Vec<PaneInfo>>,
    is_renaming: bool,
    max_len: usize,
    /// Last name we set per tab position — used to detect plugin-owned tabs and to dedupe calls.
    last_tab_name: HashMap<usize, String>,
    /// Last name we set per pane id — dedupe only.
    last_pane_name: HashMap<u32, String>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, config: BTreeMap<String, String>) {
        self.max_len = config
            .get("max_length")
            .and_then(|v| v.parse().ok())
            .filter(|n: &usize| *n >= 2)
            .unwrap_or(20);

        request_permission(&[
            PermissionType::ChangeApplicationState,
            PermissionType::ReadApplicationState,
        ]);
        subscribe(&[
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::ModeUpdate,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::ModeUpdate(info) => {
                self.is_renaming = info.mode == InputMode::RenameTab;
            }
            Event::TabUpdate(tabs) => {
                self.tabs = tabs;
                self.gc();
                self.refresh();
            }
            Event::PaneUpdate(manifest) => {
                self.panes = manifest.panes;
                self.refresh();
            }
            _ => {}
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}

impl State {
    fn refresh(&mut self) {
        if self.is_renaming {
            return;
        }
        for i in 0..self.tabs.len() {
            let tab_pos = self.tabs[i].position;
            let tab_name = self.tabs[i].name.clone();
            let Some((pane_id, new_name)) = self.derive_for_tab(tab_pos) else {
                continue;
            };
            self.set_tab_name(tab_pos, &tab_name, &new_name);
            self.set_pane_name(pane_id, &new_name);
        }
    }

    fn derive_for_tab(&self, tab_pos: usize) -> Option<(u32, String)> {
        let pane = self.focused_pane(tab_pos)?;
        let name = derive_name(&pane.title, self.max_len)?;
        Some((pane.id, name))
    }

    fn focused_pane(&self, tab_pos: usize) -> Option<&PaneInfo> {
        let panes = self.panes.get(&tab_pos)?;
        panes
            .iter()
            .find(|p| p.is_focused && !p.is_plugin)
            .or_else(|| panes.iter().find(|p| !p.is_plugin))
    }

    fn set_tab_name(&mut self, position: usize, current: &str, new_name: &str) {
        let stripped = strip_status_suffixes(current);
        let owned = self.last_tab_name.get(&position).map(String::as_str) == Some(stripped);
        // Skip user-named tabs we don't own.
        if !is_default_name(stripped) && !owned {
            return;
        }
        if owned && stripped == new_name {
            return;
        }
        // rename_tab uses 1-indexed positions; TabInfo.position is 0-indexed.
        rename_tab((position as u32) + 1, new_name);
        self.last_tab_name.insert(position, new_name.to_string());
    }

    fn set_pane_name(&mut self, pane_id: u32, new_name: &str) {
        if self.last_pane_name.get(&pane_id).map(String::as_str) == Some(new_name) {
            return;
        }
        rename_terminal_pane(pane_id, new_name);
        self.last_pane_name.insert(pane_id, new_name.to_string());
    }

    /// Drop cached entries for tabs/panes that no longer exist.
    fn gc(&mut self) {
        let live_tabs: BTreeSet<usize> = self.tabs.iter().map(|t| t.position).collect();
        self.last_tab_name.retain(|k, _| live_tabs.contains(k));

        let live_panes: BTreeSet<u32> = self.panes.values().flatten().map(|p| p.id).collect();
        self.last_pane_name.retain(|k, _| live_panes.contains(k));
    }
}

// ----- pure helpers (easy to unit-test) -----

fn derive_name(title: &str, max_len: usize) -> Option<String> {
    let raw = extract_name(title)?;
    let truncated = truncate(&raw, max_len);
    (!truncated.is_empty()).then_some(truncated)
}

/// Strip transient suffixes Zellij appends (e.g. ` (Sync)`, ` (FULLSCREEN)`).
fn strip_status_suffixes(name: &str) -> &str {
    let mut s = name.trim();
    while s.ends_with(')') {
        let Some(open) = s.rfind(" (") else { break };
        s = s[..open].trim_end();
    }
    s
}

fn is_default_name(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    name.strip_prefix("Tab #")
        .and_then(|rest| rest.parse::<usize>().ok())
        .is_some()
}

/// Convert a pane title into a short label.
/// Path-like → basename. `user@host:path` → basename of path. Anything else → as-is.
fn extract_name(title: &str) -> Option<String> {
    let t = title.trim();
    if t.is_empty() {
        return None;
    }

    let cleaned = match t.find(':') {
        Some(i) if t[..i].contains('@') => t[i + 1..].trim(),
        _ => t,
    };
    if cleaned.is_empty() {
        return None;
    }

    if cleaned.starts_with('/') || cleaned.starts_with('~') || cleaned.contains('/') {
        let trimmed = cleaned.trim_end_matches('/');
        if trimmed.is_empty() || trimmed == "~" {
            return Some("~".to_string());
        }
        return trimmed
            .rsplit('/')
            .next()
            .map(str::to_string)
            .filter(|s| !s.is_empty());
    }

    Some(cleaned.to_string())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}
