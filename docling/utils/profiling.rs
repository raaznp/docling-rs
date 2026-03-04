use std::collections::HashMap;
use std::time::Instant;

/// Performance profiling utility.
pub struct TimeRecorder {
    pub timings: HashMap<String, f64>,
}

impl TimeRecorder {
    pub fn new() -> Self {
        Self {
            timings: HashMap::new(),
        }
    }

    pub fn record(&mut self, key: &str, duration: f64) {
        let entry = self.timings.entry(key.to_string()).or_insert(0.0);
        *entry += duration;
    }

    pub fn start(&self) -> Instant {
        Instant::now()
    }
}
