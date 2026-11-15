use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock, PoisonError};

use crate::timing;

// the backend sets this once, so services never name the event loop's types
static WAKE: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

/*
 * which services changed since the windows last looked, and whether
 * something changed that no service names, which every window may show
 */
static CHANGED: Mutex<Changes> = Mutex::new(Changes {
    services: None,
    everything: false,
});

struct Changes {
    services: Option<HashSet<TypeId>>,
    everything: bool,
}

// what the window being drawn right now has read
thread_local! {
    static READ: RefCell<HashSet<TypeId>> = RefCell::new(HashSet::new());
}

pub fn set_waker(wake: impl Fn() + Send + Sync + 'static) {
    if WAKE.set(Box::new(wake)).is_err() {
        panic!("failed to set wake: already set");
    }
}

// asks the event loop to draw every window again, from any thread
pub fn mark_all() {
    if timing::enabled() {
        eprintln!("change everything");
    }

    CHANGED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .everything = true;

    ping();
}

// only the windows that read this service are drawn again
pub fn mark(service: TypeId) {
    let mut changes = CHANGED.lock().unwrap_or_else(PoisonError::into_inner);

    changes
        .services
        .get_or_insert_with(HashSet::new)
        .insert(service);

    drop(changes);

    ping();
}

// none means every window should draw
pub fn take() -> Option<HashSet<TypeId>> {
    let mut changes = CHANGED.lock().unwrap_or_else(PoisonError::into_inner);

    let services = changes.services.take().unwrap_or_default();

    if std::mem::take(&mut changes.everything) {
        return None;
    }

    Some(services)
}

pub fn note_read(service: TypeId) {
    READ.with_borrow_mut(|read| read.insert(service));
}

// what was read since the last call, which starts a fresh list
pub fn take_read() -> HashSet<TypeId> {
    READ.take()
}

fn ping() {
    // a write before the loop exists is picked up by the first frame anyway
    let Some(wake) = WAKE.get() else {
        return;
    };

    wake();
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    struct ServiceA;
    struct ServiceB;

    #[test]
    fn test_mark_and_take() {
        let _lock = TEST_MUTEX.lock().unwrap();
        // Clear any leftover state
        let _ = take();

        let id_a = TypeId::of::<ServiceA>();
        let id_b = TypeId::of::<ServiceB>();

        mark(id_a);
        let changes = take().expect("expected Some(changes)");
        assert!(changes.contains(&id_a));
        assert!(!changes.contains(&id_b));

        // After take, changes should be empty
        let empty_changes = take().expect("expected Some after take");
        assert!(empty_changes.is_empty());
    }

    #[test]
    fn test_mark_all_returns_none() {
        let _lock = TEST_MUTEX.lock().unwrap();
        // Clear any leftover state
        let _ = take();

        mark_all();
        let changes = take();
        assert!(changes.is_none());

        // Subsequent take without mark_all should return Some(empty)
        let empty_changes = take().expect("expected Some after take");
        assert!(empty_changes.is_empty());
    }

    #[test]
    fn test_read_tracking() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let id_a = TypeId::of::<ServiceA>();
        note_read(id_a);
        let read_set = take_read();
        assert!(read_set.contains(&id_a));

        let empty_set = take_read();
        assert!(empty_set.is_empty());
    }
}
