//! Cross-platform bookkeeping of which windows are attached/pinned.
//! Pure Rust so it is unit-testable without any OS calls; the Win32
//! pinner keeps its own WNDPROC map as the low-level mechanism, this is
//! the plugin-level source of truth.

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Default, Clone, Copy)]
struct Flags {
    attached: bool,
    pinned: bool,
}

#[derive(Debug, Default)]
pub(crate) struct WindowStates(Mutex<HashMap<String, Flags>>);

impl WindowStates {
    /// Returns true if the value changed.
    pub fn set_attached(&self, label: &str, attached: bool) -> bool {
        let mut map = self.0.lock().unwrap();
        let flags = map.entry(label.to_string()).or_default();
        let changed = flags.attached != attached;
        flags.attached = attached;
        changed
    }

    /// Returns true if the value changed.
    pub fn set_pinned(&self, label: &str, pinned: bool) -> bool {
        let mut map = self.0.lock().unwrap();
        let flags = map.entry(label.to_string()).or_default();
        let changed = flags.pinned != pinned;
        flags.pinned = pinned;
        changed
    }

    pub fn is_attached(&self, label: &str) -> bool {
        let map = self.0.lock().unwrap();
        map.get(label).map(|f| f.attached).unwrap_or(false)
    }

    pub fn is_pinned(&self, label: &str) -> bool {
        let map = self.0.lock().unwrap();
        map.get(label).map(|f| f.pinned).unwrap_or(false)
    }

    pub fn remove(&self, label: &str) {
        let mut map = self.0.lock().unwrap();
        map.remove(label);
    }

    pub fn attached_labels(&self) -> Vec<String> {
        let map = self.0.lock().unwrap();
        map.iter()
            .filter(|(_, f)| f.attached)
            .map(|(label, _)| label.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_not_attached_not_pinned() {
        let states = WindowStates::default();
        assert!(!states.is_attached("main"));
        assert!(!states.is_pinned("main"));
        assert!(states.attached_labels().is_empty());
    }

    #[test]
    fn tracks_attached_and_pinned_independently() {
        let states = WindowStates::default();

        assert!(states.set_attached("wallpaper", true));
        assert!(states.set_pinned("pin", true));

        assert!(states.is_attached("wallpaper"));
        assert!(!states.is_pinned("wallpaper"));
        assert!(states.is_pinned("pin"));
        assert!(!states.is_attached("pin"));
        assert_eq!(states.attached_labels(), vec!["wallpaper".to_string()]);
    }

    #[test]
    fn set_reports_change() {
        let states = WindowStates::default();
        assert!(states.set_attached("w", true));
        assert!(!states.set_attached("w", true));
        assert!(states.set_attached("w", false));
        assert!(!states.set_attached("w", false));
    }

    #[test]
    fn remove_clears_all_flags() {
        let states = WindowStates::default();
        states.set_attached("w", true);
        states.set_pinned("w", true);
        states.remove("w");
        assert!(!states.is_attached("w"));
        assert!(!states.is_pinned("w"));
    }
}
