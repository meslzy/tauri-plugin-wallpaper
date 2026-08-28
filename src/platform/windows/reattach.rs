//! Auto re-attach after explorer.exe restarts.
//!
//! When explorer restarts it destroys WorkerW, silently dropping every
//! wallpaper window off the desktop. The shell broadcasts the registered
//! `TaskbarCreated` message when the new taskbar (and desktop) comes up;
//! the helper window receives it and triggers re-attachment of every
//! window we know is attached.
//!
//! The helper WNDPROC must never block, so `on_taskbar_created` only
//! clones the callback and hands it to a fresh thread. The callback (set
//! up in `desktop::init`) retries because the desktop takes a while to
//! rebuild after the broadcast.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use crate::models::AttachOptions;

pub(crate) type ReattachFn = Arc<dyn Fn() + Send + Sync + 'static>;

static REATTACH_CALLBACK: LazyLock<Mutex<Option<ReattachFn>>> =
    LazyLock::new(|| Mutex::new(None));
static ATTACHED: LazyLock<Mutex<HashMap<String, AttachOptions>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static REATTACH_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Registered once per app in `desktop::init`; overwrite-on-reinit is fine.
pub(crate) fn set_callback(callback: ReattachFn) {
    *REATTACH_CALLBACK.lock().unwrap() = Some(callback);
}

pub(super) fn note_attached(label: &str, options: AttachOptions) {
    ATTACHED.lock().unwrap().insert(label.to_string(), options);
}

pub(crate) fn note_detached(label: &str) {
    ATTACHED.lock().unwrap().remove(label);
}

pub(crate) fn attached() -> Vec<(String, AttachOptions)> {
    ATTACHED
        .lock()
        .unwrap()
        .iter()
        .map(|(label, options)| (label.clone(), *options))
        .collect()
}

/// Clears the in-progress flag once a retry pass finishes.
fn reattach_done() {
    REATTACH_IN_PROGRESS.store(false, Ordering::SeqCst);
}

/// Helper WNDPROC entry point — must not block.
pub(super) fn on_taskbar_created() {
    if ATTACHED.lock().unwrap().is_empty() {
        return;
    }

    // Explorer restarts can broadcast in bursts; collapse them.
    if REATTACH_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    let callback = REATTACH_CALLBACK.lock().unwrap().clone();
    match callback {
        Some(callback) => {
            std::thread::spawn(move || {
                callback();
                reattach_done();
            });
        }
        None => reattach_done(),
    }
}
