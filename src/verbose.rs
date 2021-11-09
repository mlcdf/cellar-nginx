use std::sync::atomic::{AtomicUsize, Ordering};

static LEVEL: AtomicUsize = AtomicUsize::new(0);

pub fn enable() {
    if LEVEL.load(Ordering::SeqCst) != 0 {
        return;
    }
    LEVEL.fetch_add(1, Ordering::SeqCst);
}

pub fn is_enabled() -> bool {
    if LEVEL.load(Ordering::SeqCst) != 0 {
        true
    } else {
        false
    }
}
