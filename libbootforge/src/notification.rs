//! Portable wake-source contract for native device notifications.
//!
//! Wake sources only signal that observable device state may have changed. They never decide
//! event semantics. The forensic and driver comparators remain the source of truth after every
//! wake, preventing duplicated or platform-specific event meaning.

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Why a watcher was asked to rescan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeReason {
    NativeNotification,
    PollInterval,
    Manual,
    Timeout,
}

/// Result returned by a wake source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WakeResult {
    pub reason: WakeReason,
    pub coalesced_notifications: u64,
}

/// Passive source that wakes a watcher when device state may have changed.
pub trait NotificationWake: Send + Sync {
    fn wait(&self, timeout: Duration) -> WakeResult;
}

/// Thread-safe signal that native callbacks may trigger without running enumeration inside the
/// callback itself. Multiple callbacks are coalesced into one rescan with a retained count.
#[derive(Debug, Clone, Default)]
pub struct NotificationSignal {
    inner: Arc<(Mutex<SignalState>, Condvar)>,
}

#[derive(Debug, Default)]
struct SignalState {
    pending: u64,
}

impl NotificationSignal {
    /// Signal that one native notification occurred.
    pub fn notify(&self) {
        let (lock, condvar) = &*self.inner;
        let mut state = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        state.pending = state.pending.saturating_add(1);
        condvar.notify_all();
    }

    /// Consume any currently pending callbacks without blocking.
    pub fn drain(&self) -> u64 {
        let (lock, _) = &*self.inner;
        let mut state = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let pending = state.pending;
        state.pending = 0;
        pending
    }
}

impl NotificationWake for NotificationSignal {
    fn wait(&self, timeout: Duration) -> WakeResult {
        let deadline = Instant::now() + timeout;
        let (lock, condvar) = &*self.inner;
        let mut state = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

        while state.pending == 0 {
            let now = Instant::now();
            if now >= deadline {
                return WakeResult {
                    reason: WakeReason::Timeout,
                    coalesced_notifications: 0,
                };
            }
            let remaining = deadline.saturating_duration_since(now);
            let waited = condvar.wait_timeout(state, remaining);
            match waited {
                Ok((next, result)) => {
                    state = next;
                    if result.timed_out() && state.pending == 0 {
                        return WakeResult {
                            reason: WakeReason::Timeout,
                            coalesced_notifications: 0,
                        };
                    }
                }
                Err(poisoned) => {
                    let (next, _) = poisoned.into_inner();
                    state = next;
                }
            }
        }

        let pending = state.pending;
        state.pending = 0;
        WakeResult {
            reason: WakeReason::NativeNotification,
            coalesced_notifications: pending,
        }
    }
}

/// Portable fallback used where a native callback source is unavailable.
#[derive(Debug, Clone, Copy)]
pub struct PollingWake {
    interval: Duration,
}

impl PollingWake {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }
}

impl NotificationWake for PollingWake {
    fn wait(&self, timeout: Duration) -> WakeResult {
        std::thread::sleep(self.interval.min(timeout));
        WakeResult {
            reason: WakeReason::PollInterval,
            coalesced_notifications: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callbacks_are_coalesced_without_loss() {
        let signal = NotificationSignal::default();
        signal.notify();
        signal.notify();
        signal.notify();
        let result = signal.wait(Duration::from_millis(1));
        assert_eq!(result.reason, WakeReason::NativeNotification);
        assert_eq!(result.coalesced_notifications, 3);
        assert_eq!(signal.drain(), 0);
    }

    #[test]
    fn timeout_is_explicit() {
        let signal = NotificationSignal::default();
        let result = signal.wait(Duration::from_millis(1));
        assert_eq!(result.reason, WakeReason::Timeout);
        assert_eq!(result.coalesced_notifications, 0);
    }
}
