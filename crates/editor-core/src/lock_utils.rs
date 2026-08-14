//! CRIT-3: Mutex poisoning policy + lock helpers.
//!
//! ## Policy: log-and-recover
//!
//! All in-memory `Mutex<T>` and `RwLock<T>` in `editor-core` follow this
//! policy when they encounter poisoning:
//!
//! 1. **Log a warning** identifying the mutex by a caller-supplied name.
//! 2. **Recover the guard** via `PoisonError::into_inner()`, accepting that
//!    the data may be in an inconsistent state.
//! 3. Return the guard normally so the caller proceeds.
//!
//! ### Rationale
//!
//! - The WASM runtime is single-threaded, so poisoning cannot occur in
//!   production. The policy exists to make native tests + debugging safer.
//! - In native contexts (tests), poisoning happens when a test panics while
//!   holding the lock. The alternative (`.unwrap()`) panics the test
//!   process, which loses the rest of the test suite output.
//! - Recovering the guard lets tests continue, and the warning surfaces
//!   the underlying issue in test logs.
//!
//! ## Usage
//!
//! Replace `.lock().unwrap()` with `.lock_or_recover("scene_registry.entries")`.
//! For `RwLock`, use `.read_or_recover(name)` / `.write_or_recover(name)`.
//!
//! ## Not for: cross-process / persistent state
//!
//! If we ever hold state that is shared with external systems (OPFS, Bevy
//! resources, the network), the policy must escalate to
//! `lock_or_recover_or_panic` to surface corruption. Today, all `Mutex<T>`
//! in this crate hold in-memory caches whose worst-case inconsistency is
//! a stale read, not data loss.

use std::sync::{Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Log a poison event to the host (eprintln) or WASM (console.warn).
/// `mutex_name` is a stable identifier the operator can grep for.
#[cfg(not(target_arch = "wasm32"))]
fn log_poison_event(mutex_name: &str) {
    eprintln!(
        "[editor-core] WARNING: Mutex '{}' was poisoned; recovering (data may be inconsistent)",
        mutex_name
    );
}

#[cfg(target_arch = "wasm32")]
fn log_poison_event(mutex_name: &str) {
    web_sys::console::warn_1(
        &format!(
            "[editor-core] WARNING: Mutex '{}' was poisoned; recovering (data may be inconsistent)",
            mutex_name
        )
        .into(),
    );
}

/// Acquire a `Mutex` lock, recovering from poisoning with a logged warning.
///
/// `mutex_name` should be a stable, grep-friendly identifier (e.g.
/// `"scene_registry.entries"`).
pub fn lock_or_recover<'a, T: ?Sized>(mutex: &'a Mutex<T>, mutex_name: &str) -> MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            log_poison_event(mutex_name);
            poisoned.into_inner()
        }
    }
}

/// Acquire a `RwLock` read lock, recovering from poisoning with a logged warning.
pub fn read_or_recover<'a, T: ?Sized>(
    lock: &'a RwLock<T>,
    mutex_name: &str,
) -> RwLockReadGuard<'a, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            log_poison_event(mutex_name);
            poisoned.into_inner()
        }
    }
}

/// Acquire a `RwLock` write lock, recovering from poisoning with a logged warning.
pub fn write_or_recover<'a, T: ?Sized>(
    lock: &'a RwLock<T>,
    mutex_name: &str,
) -> RwLockWriteGuard<'a, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            log_poison_event(mutex_name);
            poisoned.into_inner()
        }
    }
}

/// Recover from a `PoisonError<MutexGuard<T>>` (returned by `.lock()`),
/// logging the event first. Equivalent to `lock_or_recover` but for callers
/// that already have the `PoisonError` in hand from a different acquisition
/// path (e.g. nested inside a function that returns `LockResult`).
pub fn recover_poison<'a, T>(
    err: PoisonError<MutexGuard<'a, T>>,
    mutex_name: &str,
) -> MutexGuard<'a, T> {
    log_poison_event(mutex_name);
    err.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn lock_or_recover_returns_guard_on_normal_lock() {
        let mutex = Mutex::new(42u32);
        let guard = lock_or_recover(&mutex, "test.normal");
        assert_eq!(*guard, 42);
    }

    #[test]
    fn lock_or_recover_recovers_after_poison() {
        let mutex = Mutex::new(String::from("initial"));

        // Poison the mutex by panicking while holding the lock.
        let result = std::panic::catch_unwind(|| {
            let _guard = mutex.lock().unwrap();
            panic!("intentional panic to poison the mutex");
        });
        assert!(result.is_err());

        // Without recovery, .lock() would return Err(PoisonError).
        // With lock_or_recover, we recover the guard.
        let guard = lock_or_recover(&mutex, "test.poisoned");
        assert_eq!(&*guard, "initial");
    }

    #[test]
    fn read_or_recover_after_poison() {
        let lock = RwLock::new(vec![1, 2, 3]);
        let _ = std::panic::catch_unwind(|| {
            let _g = lock.write().unwrap();
            panic!("poison rwlock");
        });

        let guard = read_or_recover(&lock, "test.rwlock.read");
        assert_eq!(guard.len(), 3);
    }

    #[test]
    fn write_or_recover_after_poison() {
        let lock = RwLock::new(vec![1, 2, 3]);
        let _ = std::panic::catch_unwind(|| {
            let _g = lock.write().unwrap();
            panic!("poison rwlock");
        });

        let mut guard = write_or_recover(&lock, "test.rwlock.write");
        guard.push(4);
        assert_eq!(guard.len(), 4);
    }

    #[test]
    fn lock_or_recover_with_arc_mutex() {
        let mutex = Arc::new(Mutex::new(0u32));
        let mutex_clone = Arc::clone(&mutex);

        let _ = std::panic::catch_unwind(|| {
            let _g = mutex.lock().unwrap();
            panic!("poison via Arc");
        });

        let guard = lock_or_recover(&mutex_clone, "test.arc.poisoned");
        assert_eq!(*guard, 0);
    }

    #[test]
    fn recover_poison_unwraps_with_log() {
        // recover_poison takes a PoisonError<MutexGuard<T>> (the result of
        // .lock()) and returns a MutexGuard<T>, with a logged warning. This
        // is the same recovery path as lock_or_recover, but for callers that
        // already obtained the PoisonError from a different acquisition.
        let mutex = Mutex::new(7u32);
        let _ = std::panic::catch_unwind(|| {
            let _g = mutex.lock().unwrap();
            panic!("poison for recover_poison test");
        });

        let err = mutex.lock().expect_err("should be poisoned");
        let guard = recover_poison(err, "test.recover_poison");
        assert_eq!(*guard, 7);
    }
}
