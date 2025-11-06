//! Retry logic with exponential backoff and jitter

use crate::{Error, Result};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
    /// Add jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Execute operation with retry logic
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        let mut backoff = self.initial_backoff;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        debug!("Operation succeeded after {} attempts", attempt);
                    }
                    return Ok(result);
                }
                Err(e) if e.is_retryable() && attempt < self.max_attempts => {
                    warn!(
                        "Operation failed (attempt {}/{}): {}. Retrying after {:?}",
                        attempt, self.max_attempts, e, backoff
                    );

                    sleep(backoff).await;

                    // Calculate next backoff with exponential growth
                    backoff = Duration::from_secs_f64(
                        (backoff.as_secs_f64() * self.multiplier).min(self.max_backoff.as_secs_f64())
                    );

                    // Add jitter if enabled
                    if self.jitter {
                        backoff = self.add_jitter(backoff);
                    }
                }
                Err(e) => {
                    if attempt >= self.max_attempts {
                        warn!("Operation failed after {} attempts: {}", attempt, e);
                    }
                    return Err(e);
                }
            }
        }
    }

    fn add_jitter(&self, duration: Duration) -> Duration {
        use rand::Rng;
        let jitter_ms = rand::thread_rng().gen_range(0..=duration.as_millis() / 4);
        duration + Duration::from_millis(jitter_ms as u64)
    }
}

/// Circuit breaker for preventing cascading failures
pub struct CircuitBreaker {
    failure_threshold: u32,
    consecutive_failures: std::sync::atomic::AtomicU32,
    reset_timeout: Duration,
    last_failure: parking_lot::Mutex<Option<std::time::Instant>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            consecutive_failures: std::sync::atomic::AtomicU32::new(0),
            reset_timeout,
            last_failure: parking_lot::Mutex::new(None),
        }
    }

    /// Check if circuit is open (preventing operations)
    pub fn is_open(&self) -> bool {
        let failures = self.consecutive_failures.load(std::sync::atomic::Ordering::Relaxed);
        
        if failures >= self.failure_threshold {
            let last_failure = self.last_failure.lock();
            if let Some(time) = *last_failure {
                if time.elapsed() < self.reset_timeout {
                    return true;
                } else {
                    // Reset after timeout
                    drop(last_failure);
                    self.reset();
                    return false;
                }
            }
        }
        
        false
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        self.consecutive_failures.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        self.consecutive_failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        *self.last_failure.lock() = Some(std::time::Instant::now());
    }

    /// Reset circuit breaker
    pub fn reset(&self) {
        self.consecutive_failures.store(0, std::sync::atomic::Ordering::Relaxed);
        *self.last_failure.lock() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success() {
        let policy = RetryPolicy::default();
        let mut attempts = 0;

        let result = policy
            .execute(|| async {
                attempts += 1;
                if attempts < 3 {
                    Err(Error::Timeout)
                } else {
                    Ok(42)
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let policy = RetryPolicy {
            max_attempts: 2,
            ..Default::default()
        };

        let result = policy
            .execute(|| async { Err::<(), _>(Error::Timeout) })
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(3, Duration::from_secs(1));
        
        assert!(!breaker.is_open());
        
        breaker.record_failure();
        breaker.record_failure();
        breaker.record_failure();
        
        assert!(breaker.is_open());
        
        breaker.record_success();
        assert!(!breaker.is_open());
    }
}
