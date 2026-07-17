use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentTab {
    pub id: u64,
    pub path: PathBuf,
    pub dirty: bool,
    pub missing: bool,
    pub edit_on_open: bool,
}

#[derive(Debug, Default)]
pub struct DocumentSession {
    pub tabs: Vec<DocumentTab>,
    pub active_id: Option<u64>,
    next_id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct PersistedSession {
    version: u8,
    active: Option<usize>,
    tabs: Vec<PathBuf>,
}

impl DocumentSession {
    pub fn load(path: &Path) -> Self {
        let Ok(raw) = fs::read(path) else {
            return Self::default();
        };
        let Ok(saved) = serde_json::from_slice::<PersistedSession>(&raw) else {
            return Self::default();
        };
        if saved.version != 1 {
            return Self::default();
        }

        let mut session = Self::default();
        for path in saved.tabs {
            session.open(path, false);
        }
        session.active_id = saved
            .active
            .and_then(|index| session.tabs.get(index))
            .map(|tab| tab.id)
            .or_else(|| session.tabs.last().map(|tab| tab.id));
        session
    }

    pub fn open(&mut self, path: PathBuf, edit_on_open: bool) -> u64 {
        let path = normalize_path(path);
        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.path == path) {
            tab.missing = !tab.path.exists();
            tab.edit_on_open |= edit_on_open;
            self.active_id = Some(tab.id);
            return tab.id;
        }

        self.next_id += 1;
        let id = self.next_id;
        self.tabs.push(DocumentTab {
            id,
            missing: !path.exists(),
            path,
            dirty: false,
            edit_on_open,
        });
        self.active_id = Some(id);
        id
    }

    pub fn activate(&mut self, id: u64) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) else {
            return false;
        };
        tab.missing = !tab.path.exists();
        self.active_id = Some(id);
        true
    }

    pub fn close(&mut self, id: u64) -> bool {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return false;
        };
        let was_active = self.active_id == Some(id);
        self.tabs.remove(index);
        if was_active {
            self.active_id = self
                .tabs
                .get(index)
                .or_else(|| index.checked_sub(1).and_then(|i| self.tabs.get(i)))
                .map(|tab| tab.id);
        }
        true
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let active = self
            .active_id
            .and_then(|id| self.tabs.iter().position(|tab| tab.id == id));
        let saved = PersistedSession {
            version: 1,
            active,
            tabs: self.tabs.iter().map(|tab| tab.path.clone()).collect(),
        };
        let body = serde_json::to_vec_pretty(&saved).map_err(io::Error::other)?;
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, body)?;
        #[cfg(target_os = "windows")]
        if path.exists() {
            fs::remove_file(path)?;
        }
        fs::rename(temporary, path)
    }

    pub fn active(&self) -> Option<&DocumentTab> {
        let id = self.active_id?;
        self.tabs.iter().find(|tab| tab.id == id)
    }

    pub fn active_mut(&mut self) -> Option<&mut DocumentTab> {
        let id = self.active_id?;
        self.tabs.iter_mut().find(|tab| tab.id == id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut DocumentTab> {
        self.tabs.iter_mut().find(|tab| tab.id == id)
    }

    pub fn relocate(&mut self, id: u64, path: PathBuf) -> bool {
        let path = normalize_path(path);
        if self.tabs.iter().any(|tab| tab.id != id && tab.path == path) {
            return false;
        }
        let Some(tab) = self.get_mut(id) else {
            return false;
        };
        tab.path = path;
        tab.missing = !tab.path.exists();
        true
    }
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let absolute = if path.is_absolute() {
        path
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    fs::canonicalize(&absolute).unwrap_or_else(|_| {
        absolute
            .parent()
            .and_then(|parent| fs::canonicalize(parent).ok())
            .and_then(|parent| absolute.file_name().map(|name| parent.join(name)))
            .unwrap_or(absolute)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "md-preview-session-{name}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn opening_the_same_path_activates_without_duplicate() {
        let dir = temp_dir("dedupe");
        let file = dir.join("note.md");
        fs::write(&file, "# Note").unwrap();
        let mut session = DocumentSession::default();

        let first = session.open(file.clone(), false);
        let second = session.open(file.clone(), true);

        assert_eq!(first, second);
        assert_eq!(session.tabs.len(), 1);
        assert_eq!(session.active_id, Some(first));
        assert!(session.tabs[0].edit_on_open);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn closing_active_tab_selects_the_next_neighbor() {
        let dir = temp_dir("close");
        let mut session = DocumentSession::default();
        let first = session.open(dir.join("one.md"), false);
        let second = session.open(dir.join("two.md"), false);
        let third = session.open(dir.join("three.md"), false);
        assert!(session.activate(second));

        assert!(session.close(second));

        assert_eq!(session.active_id, Some(third));
        assert_eq!(
            session.tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
            vec![first, third]
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn persisted_session_keeps_missing_tabs_and_active_order() {
        let dir = temp_dir("roundtrip");
        let state_path = dir.join("session.json");
        let existing = dir.join("existing.md");
        let missing = dir.join("missing.md");
        fs::write(&existing, "# Existing").unwrap();
        let mut session = DocumentSession::default();
        session.open(existing.clone(), false);
        session.open(missing.clone(), false);

        session.save(&state_path).unwrap();
        let restored = DocumentSession::load(&state_path);

        assert_eq!(restored.tabs.len(), 2);
        assert_eq!(restored.tabs[0].path, fs::canonicalize(existing).unwrap());
        assert_eq!(
            restored.tabs[1].path,
            fs::canonicalize(&dir)
                .unwrap()
                .join(missing.file_name().unwrap())
        );
        assert!(!restored.tabs[0].missing);
        assert!(restored.tabs[1].missing);
        assert_eq!(restored.active_id, restored.tabs.get(1).map(|tab| tab.id));
        let _ = fs::remove_dir_all(dir);
    }
}
