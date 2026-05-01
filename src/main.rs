use std::collections::{BTreeMap, HashMap};
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    tabs: Vec<TabInfo>,
    panes: HashMap<usize, Vec<PaneInfo>>,
    is_renaming: bool,
    max_len: usize,
    /// Last name we set for each tab position, to avoid redundant rename calls
    last_set: HashMap<usize, String>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, config: BTreeMap<String, String>) {
        self.max_len = config
            .get("max_length")
            .and_then(|v| v.parse().ok())
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
            Event::ModeUpdate(mode_info) => {
                self.is_renaming = mode_info.mode == InputMode::RenameTab;
            }
            Event::TabUpdate(tabs) => {
                self.tabs = tabs;
                self.rename_tabs();
            }
            Event::PaneUpdate(manifest) => {
                self.panes = manifest.panes;
                self.rename_tabs();
            }
            _ => {}
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}

impl State {
    fn rename_tabs(&mut self) {
        if self.is_renaming {
            return;
        }

        for tab in &self.tabs {
            // Only rename tabs with default names or names we previously set
            if !is_default_name(&tab.name) {
                if self.last_set.get(&tab.position) != Some(&tab.name) {
                    continue;
                }
            }

            if let Some(name) = self.derive_name(tab) {
                // Skip if we already set this exact name for this position
                if self.last_set.get(&tab.position) == Some(&name) {
                    continue;
                }
                rename_tab(tab.position as u32, &name);
                self.last_set.insert(tab.position, name);
            }
        }
    }

    fn derive_name(&self, tab: &TabInfo) -> Option<String> {
        let panes = self.panes.get(&tab.position)?;
        let focused = panes
            .iter()
            .find(|p| p.is_focused && !p.is_plugin)
            .or_else(|| panes.iter().find(|p| !p.is_plugin))?;

        let raw = extract_name(&focused.title)?;
        let name = truncate(&raw, self.max_len);
        if name.is_empty() {
            return None;
        }
        Some(name)
    }
}

fn is_default_name(name: &str) -> bool {
    name.starts_with("Tab #") && name[5..].parse::<usize>().is_ok()
}

/// Extract a tab name from a pane title.
fn extract_name(title: &str) -> Option<String> {
    let t = title.trim();
    if t.is_empty() {
        return None;
    }

    // Handle user@host:path format
    let cleaned = if let Some(colon_pos) = t.find(':') {
        if t[..colon_pos].contains('@') {
            &t[colon_pos + 1..]
        } else {
            t
        }
    } else {
        t
    };

    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return None;
    }

    // If it looks like a path, extract basename
    if cleaned.contains('/') || cleaned.starts_with('~') {
        let path = cleaned.trim_end_matches('/');
        if path.is_empty() || path == "~" {
            return Some("~".to_string());
        }
        return path.rsplit('/').next().map(|s| s.to_string()).filter(|s| !s.is_empty());
    }

    // Otherwise use as-is (program name, etc.)
    Some(cleaned.to_string())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
