use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::HttpRetryConfig;

pub fn jittered_delay_for_retry(config: HttpRetryConfig, retry_index: u32) -> Duration {
    let base = config.delay_for_retry(retry_index);
    if base.is_zero() {
        return base;
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as u64)
        .unwrap_or(0);
    let jitter_ms = nanos % 100;
    base + Duration::from_millis(jitter_ms)
}

#[cfg(test)]
mod tests {
    use super::jittered_delay_for_retry;
    use crate::HttpRetryConfig;

    #[test]
    fn jittered_delay_is_at_least_base_delay() {
        let config = HttpRetryConfig {
            max_attempts: 3,
            base_delay_ms: 200,
            max_delay_ms: 400,
        };

        assert!(jittered_delay_for_retry(config, 0) >= std::time::Duration::from_millis(200));
    }
}
