use std::time::{Duration, Instant};

/// Valve no documenta el límite de `appdetails`; la cifra que circula es ~200 peticiones
/// cada 5 minutos por IP. Se va por debajo (1 cada 1,5 s) y ante un 429/403 se espera
/// con retroceso exponencial.
const MIN_INTERVAL: Duration = Duration::from_millis(1500);
const FIRST_BACKOFF: Duration = Duration::from_secs(30);
const MAX_BACKOFF: Duration = Duration::from_secs(600);

#[derive(Debug)]
pub struct RateLimiter {
    next_allowed: Option<Instant>,
    backoff: Option<Duration>,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter { next_allowed: None, backoff: None }
    }

    /// Cuánto hay que esperar antes de la siguiente petición.
    pub fn delay(&self, now: Instant) -> Duration {
        self.next_allowed.map_or(Duration::ZERO, |t| t.saturating_duration_since(now))
    }

    pub fn on_success(&mut self, now: Instant) {
        self.backoff = None;
        self.next_allowed = Some(now + MIN_INTERVAL);
    }

    pub fn on_throttled(&mut self, now: Instant) {
        let backoff = self.backoff.map_or(FIRST_BACKOFF, |b| (b * 2).min(MAX_BACKOFF));
        self.backoff = Some(backoff);
        self.next_allowed = Some(now + backoff);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spaces_requests_and_backs_off_exponentially_up_to_a_cap() {
        let t0 = Instant::now();
        let mut limiter = RateLimiter::new();
        assert_eq!(limiter.delay(t0), Duration::ZERO);

        limiter.on_success(t0);
        assert_eq!(limiter.delay(t0), MIN_INTERVAL);

        limiter.on_throttled(t0);
        assert_eq!(limiter.delay(t0), FIRST_BACKOFF);
        limiter.on_throttled(t0);
        assert_eq!(limiter.delay(t0), FIRST_BACKOFF * 2);
        for _ in 0..10 {
            limiter.on_throttled(t0);
        }
        assert_eq!(limiter.delay(t0), MAX_BACKOFF);

        limiter.on_success(t0);
        assert_eq!(limiter.delay(t0), MIN_INTERVAL);
    }
}
