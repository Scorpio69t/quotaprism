use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub struct ManualClock {
    now: Mutex<DateTime<Utc>>,
}

impl ManualClock {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    pub fn set(&self, now: DateTime<Utc>) {
        *self.now.lock().expect("manual clock lock poisoned") = now;
    }

    pub fn advance(&self, duration: Duration) {
        let mut now = self.now.lock().expect("manual clock lock poisoned");
        *now += duration;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().expect("manual clock lock poisoned")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshErrorKind {
    Auth,
    Permission,
    RateLimit,
    Network,
    Timeout,
    Schema,
    Unsupported,
    Storage,
    Cancelled,
    Unknown,
    Skipped,
}

#[derive(Clone, Debug)]
pub struct RefreshError {
    pub kind: RefreshErrorKind,
    pub message: String,
    pub retry_after: Option<Duration>,
}

impl RefreshError {
    pub fn auth(message: impl Into<String>) -> Self {
        Self {
            kind: RefreshErrorKind::Auth,
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn permission(message: impl Into<String>) -> Self {
        Self {
            kind: RefreshErrorKind::Permission,
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn rate_limit(retry_after: Duration, message: impl Into<String>) -> Self {
        Self {
            kind: RefreshErrorKind::RateLimit,
            message: message.into(),
            retry_after: Some(retry_after),
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self {
            kind: RefreshErrorKind::Network,
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self {
            kind: RefreshErrorKind::Timeout,
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn schema(message: impl Into<String>) -> Self {
        Self {
            kind: RefreshErrorKind::Schema,
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            kind: RefreshErrorKind::Unsupported,
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn storage(message: impl Into<String>) -> Self {
        Self {
            kind: RefreshErrorKind::Storage,
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn cancelled() -> Self {
        Self {
            kind: RefreshErrorKind::Cancelled,
            message: "Refresh cancelled".into(),
            retry_after: None,
        }
    }

    pub fn unknown(message: impl Into<String>) -> Self {
        Self {
            kind: RefreshErrorKind::Unknown,
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn skipped(message: impl Into<String>) -> Self {
        Self {
            kind: RefreshErrorKind::Skipped,
            message: message.into(),
            retry_after: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RefreshOutcome {
    pub account_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub success: bool,
    pub error: Option<RefreshError>,
    pub retry_after: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Default)]
pub struct RefreshState {
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub error_kind: Option<RefreshErrorKind>,
    pub retry_after: Option<DateTime<Utc>>,
    pub in_flight: bool,
    pub failure_count: u32,
}

#[derive(Clone, Default)]
pub struct RefreshControl {
    cancelled: Arc<AtomicBool>,
}

impl RefreshControl {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

struct InFlightResult {
    done: bool,
    outcome: Option<RefreshOutcome>,
}

struct InFlightHandle {
    result: Mutex<InFlightResult>,
    condvar: Condvar,
}

impl InFlightHandle {
    fn new() -> Self {
        Self {
            result: Mutex::new(InFlightResult {
                done: false,
                outcome: None,
            }),
            condvar: Condvar::new(),
        }
    }

    fn wait(&self) -> RefreshOutcome {
        let mut result = self.result.lock().expect("in-flight result lock poisoned");
        while !result.done {
            result = self
                .condvar
                .wait(result)
                .expect("in-flight result condvar poisoned");
        }
        result
            .outcome
            .clone()
            .expect("completed in-flight refresh must have an outcome")
    }

    fn complete(&self, outcome: RefreshOutcome) {
        let mut result = self.result.lock().expect("in-flight result lock poisoned");
        result.done = true;
        result.outcome = Some(outcome);
        self.condvar.notify_all();
    }
}

#[derive(Default)]
struct SchedulerData {
    states: HashMap<String, RefreshState>,
    in_flight: HashMap<String, Arc<InFlightHandle>>,
}

pub struct RefreshScheduler {
    clock: Arc<dyn Clock>,
    data: Mutex<SchedulerData>,
}

impl RefreshScheduler {
    pub fn system() -> Self {
        Self::new(Arc::new(SystemClock))
    }

    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            data: Mutex::new(SchedulerData::default()),
        }
    }

    pub fn state(&self, account_id: &str) -> RefreshState {
        self.data
            .lock()
            .expect("scheduler state lock poisoned")
            .states
            .get(account_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn refresh<F>(
        &self,
        account_id: &str,
        control: &RefreshControl,
        action: F,
    ) -> RefreshOutcome
    where
        F: FnOnce() -> Result<(), RefreshError>,
    {
        let now = self.clock.now();
        let handle = {
            let mut data = self.data.lock().expect("scheduler state lock poisoned");
            if let Some(handle) = data.in_flight.get(account_id).cloned() {
                drop(data);
                return handle.wait();
            }

            let state = data.states.get(account_id).cloned().unwrap_or_default();
            if let Some(retry_after) = state.retry_after {
                if now < retry_after {
                    return skipped_outcome(account_id, now, retry_after);
                }
            }

            let handle = Arc::new(InFlightHandle::new());
            data.in_flight
                .insert(account_id.to_string(), handle.clone());
            data.states
                .entry(account_id.to_string())
                .or_default()
                .in_flight = true;
            handle
        };

        let started_at = self.clock.now();
        let outcome = if control.is_cancelled() {
            let error = RefreshError::cancelled();
            failed_outcome(account_id, started_at, self.clock.now(), error, None)
        } else {
            match action() {
                Ok(()) => RefreshOutcome {
                    account_id: account_id.to_string(),
                    started_at,
                    completed_at: self.clock.now(),
                    success: true,
                    error: None,
                    retry_after: None,
                },
                Err(error) => {
                    let retry_after = self.retry_after(account_id, &error);
                    failed_outcome(account_id, started_at, self.clock.now(), error, retry_after)
                }
            }
        };

        self.complete(account_id, &outcome);
        handle.complete(outcome.clone());
        outcome
    }

    fn retry_after(&self, account_id: &str, error: &RefreshError) -> Option<DateTime<Utc>> {
        let now = self.clock.now();
        match error.kind {
            RefreshErrorKind::Auth
            | RefreshErrorKind::Permission
            | RefreshErrorKind::Schema
            | RefreshErrorKind::Unsupported
            | RefreshErrorKind::Storage
            | RefreshErrorKind::Cancelled
            | RefreshErrorKind::Skipped => None,
            RefreshErrorKind::RateLimit => {
                Some(now + error.retry_after.unwrap_or(Duration::minutes(1)))
            }
            RefreshErrorKind::Network | RefreshErrorKind::Timeout | RefreshErrorKind::Unknown => {
                let data = self.data.lock().expect("scheduler state lock poisoned");
                let failure_count = data
                    .states
                    .get(account_id)
                    .map(|state| state.failure_count)
                    .unwrap_or(0)
                    .saturating_add(1);
                drop(data);
                let exponent = failure_count.saturating_sub(1).min(6);
                let seconds = 15_i64 * 2_i64.pow(exponent);
                Some(now + Duration::seconds(seconds))
            }
        }
    }

    fn complete(&self, account_id: &str, outcome: &RefreshOutcome) {
        let mut data = self.data.lock().expect("scheduler state lock poisoned");
        data.in_flight.remove(account_id);
        let state = data.states.entry(account_id.to_string()).or_default();
        state.in_flight = false;
        state.last_attempt_at = Some(outcome.started_at);
        state.retry_after = outcome.retry_after;
        if outcome.success {
            state.last_success_at = Some(outcome.completed_at);
            state.error_kind = None;
            state.failure_count = 0;
        } else {
            state.error_kind = outcome.error.as_ref().map(|error| error.kind);
            if matches!(
                outcome.error.as_ref().map(|error| error.kind),
                Some(
                    RefreshErrorKind::Network
                        | RefreshErrorKind::Timeout
                        | RefreshErrorKind::Unknown
                )
            ) {
                state.failure_count = state.failure_count.saturating_add(1);
            } else if matches!(
                outcome.error.as_ref().map(|error| error.kind),
                Some(
                    RefreshErrorKind::Auth
                        | RefreshErrorKind::Permission
                        | RefreshErrorKind::Schema
                        | RefreshErrorKind::Unsupported
                        | RefreshErrorKind::Storage
                        | RefreshErrorKind::Cancelled
                )
            ) {
                state.failure_count = 0;
            }
        }
    }
}

fn skipped_outcome(
    account_id: &str,
    now: DateTime<Utc>,
    retry_after: DateTime<Utc>,
) -> RefreshOutcome {
    RefreshOutcome {
        account_id: account_id.to_string(),
        started_at: now,
        completed_at: now,
        success: false,
        error: Some(RefreshError::skipped("Refresh is cooling down")),
        retry_after: Some(retry_after),
    }
}

fn failed_outcome(
    account_id: &str,
    started_at: DateTime<Utc>,
    completed_at: DateTime<Utc>,
    error: RefreshError,
    retry_after: Option<DateTime<Utc>>,
) -> RefreshOutcome {
    RefreshOutcome {
        account_id: account_id.to_string(),
        started_at,
        completed_at,
        success: false,
        error: Some(error),
        retry_after,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration as StdDuration;

    fn at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 27, 2, 0, 0).unwrap()
    }

    #[test]
    fn auth_failure_has_no_retry_cooldown() {
        let clock = Arc::new(ManualClock::new(at()));
        let scheduler = RefreshScheduler::new(clock.clone());
        let outcome = scheduler.refresh("account-1", &RefreshControl::default(), || {
            Err(RefreshError::auth("401"))
        });
        assert_eq!(
            outcome.error.as_ref().map(|error| error.kind),
            Some(RefreshErrorKind::Auth)
        );
        assert_eq!(scheduler.state("account-1").retry_after, None);
    }

    #[test]
    fn rate_limit_respects_retry_after() {
        let clock = Arc::new(ManualClock::new(at()));
        let scheduler = RefreshScheduler::new(clock.clone());
        let outcome = scheduler.refresh("account-1", &RefreshControl::default(), || {
            Err(RefreshError::rate_limit(Duration::minutes(5), "429"))
        });
        assert_eq!(outcome.retry_after, Some(at() + Duration::minutes(5)));

        let calls = Arc::new(AtomicBool::new(false));
        clock.advance(Duration::minutes(4));
        let skipped = scheduler.refresh("account-1", &RefreshControl::default(), || {
            calls.store(true, Ordering::SeqCst);
            Ok(())
        });
        assert!(!calls.load(Ordering::SeqCst));
        assert_eq!(
            skipped.error.as_ref().map(|error| error.kind),
            Some(RefreshErrorKind::Skipped)
        );

        clock.advance(Duration::minutes(1));
        let success = scheduler.refresh("account-1", &RefreshControl::default(), || Ok(()));
        assert!(success.success);
    }

    #[test]
    fn timeout_uses_exponential_backoff() {
        let clock = Arc::new(ManualClock::new(at()));
        let scheduler = RefreshScheduler::new(clock.clone());
        let first = scheduler.refresh("account-1", &RefreshControl::default(), || {
            Err(RefreshError::timeout("deadline"))
        });
        assert_eq!(first.retry_after, Some(at() + Duration::seconds(15)));

        clock.advance(Duration::seconds(14));
        let skipped = scheduler.refresh("account-1", &RefreshControl::default(), || Ok(()));
        assert_eq!(
            skipped.error.as_ref().map(|error| error.kind),
            Some(RefreshErrorKind::Skipped)
        );

        clock.advance(Duration::seconds(1));
        let second = scheduler.refresh("account-1", &RefreshControl::default(), || Ok(()));
        assert!(second.success);
    }

    #[test]
    fn cancellation_runs_no_action_and_is_not_retried() {
        let clock = Arc::new(ManualClock::new(at()));
        let scheduler = RefreshScheduler::new(clock);
        let control = RefreshControl::default();
        control.cancel();
        let calls = Arc::new(AtomicBool::new(false));
        let outcome = scheduler.refresh("account-1", &control, || {
            calls.store(true, Ordering::SeqCst);
            Ok(())
        });
        assert!(!calls.load(Ordering::SeqCst));
        assert_eq!(
            outcome.error.as_ref().map(|error| error.kind),
            Some(RefreshErrorKind::Cancelled)
        );
        assert_eq!(scheduler.state("account-1").retry_after, None);
    }

    #[test]
    fn same_account_refresh_is_singleflight() {
        let clock = Arc::new(ManualClock::new(at()));
        let scheduler = Arc::new(RefreshScheduler::new(clock));
        let calls = Arc::new(AtomicUsize::new(0));
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();

        let first_scheduler = scheduler.clone();
        let first_calls = calls.clone();
        let first_control = RefreshControl::default();
        let first = thread::spawn(move || {
            first_scheduler.refresh("account-1", &first_control, || {
                first_calls.fetch_add(1, Ordering::SeqCst);
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
        });

        started_rx.recv().unwrap();
        let second_scheduler = scheduler.clone();
        let second_control = RefreshControl::default();
        let second_calls = calls.clone();
        let second = thread::spawn(move || {
            second_scheduler.refresh("account-1", &second_control, || {
                second_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        });

        thread::sleep(StdDuration::from_millis(20));
        release_tx.send(()).unwrap();
        let first_outcome = first.join().unwrap();
        let second_outcome = second.join().unwrap();
        assert!(first_outcome.success);
        assert!(second_outcome.success);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn different_accounts_refresh_independently() {
        let clock = Arc::new(ManualClock::new(at()));
        let scheduler = Arc::new(RefreshScheduler::new(clock));
        let (one_started_tx, one_started_rx) = mpsc::channel();
        let (one_release_tx, one_release_rx) = mpsc::channel();
        let (two_started_tx, two_started_rx) = mpsc::channel();
        let (two_release_tx, two_release_rx) = mpsc::channel();

        let scheduler_one = scheduler.clone();
        let one = thread::spawn(move || {
            scheduler_one.refresh("account-1", &RefreshControl::default(), || {
                one_started_tx.send(()).unwrap();
                one_release_rx.recv().unwrap();
                Ok(())
            })
        });

        let scheduler_two = scheduler.clone();
        let two = thread::spawn(move || {
            scheduler_two.refresh("account-2", &RefreshControl::default(), || {
                two_started_tx.send(()).unwrap();
                two_release_rx.recv().unwrap();
                Ok(())
            })
        });

        one_started_rx.recv().unwrap();
        two_started_rx.recv().unwrap();
        one_release_tx.send(()).unwrap();
        two_release_tx.send(()).unwrap();
        assert!(one.join().unwrap().success);
        assert!(two.join().unwrap().success);
    }
}
