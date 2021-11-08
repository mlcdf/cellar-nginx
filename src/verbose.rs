use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};

static LEVEL: AtomicUsize = AtomicUsize::new(0);

/// Verbosity level option <`Verbose`|`Terse`|`Quite`>
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Verbosity {
    /// Minimal reporting option
    Quiet = 0,
    /// Extended reporting option
    Verbose = 1,
}

impl Display for Verbosity {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verbose => fmt.write_str("verbose"),
            Self::Quiet => fmt.write_str("quiet"),
        }
    }
}

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