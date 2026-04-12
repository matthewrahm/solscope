use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Simple in-memory TTL cache
#[allow(dead_code)]
pub struct Cache<T> {
    entries: HashMap<String, (Instant, T)>,
    ttl: Duration,
}

#[allow(dead_code)]
impl<T: Clone> Cache<T> {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            entries: HashMap::new(),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.entries.get(key).and_then(|(created, val)| {
            if created.elapsed() < self.ttl {
                Some(val)
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, key: String, value: T) {
        self.entries.insert(key, (Instant::now(), value));
    }

    /// Remove expired entries
    pub fn cleanup(&mut self) {
        self.entries.retain(|_, (created, _)| created.elapsed() < self.ttl);
    }
}

/// Tracks price samples over time for sparkline rendering
pub struct PriceHistory {
    /// Map of mint -> vec of (timestamp, price) samples
    samples: HashMap<String, Vec<f64>>,
    max_samples: usize,
}

impl PriceHistory {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: HashMap::new(),
            max_samples,
        }
    }

    pub fn record(&mut self, mint: &str, price: f64) {
        let entry = self.samples.entry(mint.to_string()).or_default();
        entry.push(price);
        if entry.len() > self.max_samples {
            entry.remove(0);
        }
    }

    pub fn get(&self, mint: &str) -> Option<&[f64]> {
        self.samples.get(mint).map(|v| v.as_slice())
    }
}
