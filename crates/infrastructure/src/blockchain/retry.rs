use std::thread;
use std::time::Duration;
use tracing::warn;

/// Retry a synchronous operation with exponential backoff.
///
/// `is_retryable` returns true for errors that warrant a retry.
/// Delays are `base_delay * 2^attempt` capped at `max_delay`.
pub fn with_retry<T, E, F>(
    mut operation: F,
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
    is_retryable: fn(&E) -> bool,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut last_err = None;
    for attempt in 0..max_attempts {
        match operation() {
            Ok(value) => return Ok(value),
            Err(err) => {
                if !is_retryable(&err) || attempt == max_attempts - 1 {
                    return Err(err);
                }
                let delay = base_delay.mul_f64(2f64.powi(attempt as i32)).min(max_delay);
                warn!(
                    attempt = attempt + 1,
                    delay_ms = delay.as_millis() as u64,
                    "operation failed, retrying"
                );
                last_err = Some(err);
                thread::sleep(delay);
            }
        }
    }
    Err(last_err.expect("max_attempts must be > 0"))
}

/// Default retry predicate: retry on any error.
pub fn retry_any<E>(_: &E) -> bool {
    true
}

/// Retry predicate for EVM write operations: only retry on submission failures
/// (e.g. RPC timeout), never on reverts or invalid input.
pub fn retry_on_submission_failed<E: AsRef<str>>(err: &E) -> bool {
    err.as_ref().contains("submission failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn retry_succeeds_after_transient_failures() {
        let counter = AtomicUsize::new(0);
        let result = with_retry(
            || {
                let attempt = counter.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    Err::<i32, &str>("submission failed: timeout")
                } else {
                    Ok::<i32, &str>(42)
                }
            },
            5,
            Duration::from_millis(10),
            Duration::from_millis(50),
            retry_any,
        );
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn retry_gives_up_after_max_attempts() {
        let result = with_retry(
            || Err::<(), &str>("permanent failure"),
            3,
            Duration::from_millis(1),
            Duration::from_millis(5),
            retry_any,
        );
        assert!(result.is_err());
    }

    #[test]
    fn retry_respects_non_retryable_errors() {
        let counter = AtomicUsize::new(0);
        let result = with_retry(
            || {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<(), &str>("reverted: invalid proof")
            },
            5,
            Duration::from_millis(1),
            Duration::from_millis(5),
            retry_on_submission_failed,
        );
        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
