//! Runtime counters for scans, event latency, queue pressure, and dropped work.

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerformanceMetrics {
    pub scans: u64,
    pub events_emitted: u64,
    pub dropped_events: u64,
    pub peak_queue_depth: usize,
    pub total_scan_latency_micros: u128,
    pub total_event_latency_micros: u128,
}

impl PerformanceMetrics {
    pub fn record_scan(&mut self, latency: Duration) {
        self.scans = self.scans.saturating_add(1);
        self.total_scan_latency_micros = self
            .total_scan_latency_micros
            .saturating_add(latency.as_micros());
    }

    pub fn record_event(&mut self, latency: Duration) {
        self.events_emitted = self.events_emitted.saturating_add(1);
        self.total_event_latency_micros = self
            .total_event_latency_micros
            .saturating_add(latency.as_micros());
    }

    pub fn record_queue_depth(&mut self, depth: usize) {
        self.peak_queue_depth = self.peak_queue_depth.max(depth);
    }

    pub fn record_drop(&mut self, count: u64) {
        self.dropped_events = self.dropped_events.saturating_add(count);
    }

    pub fn average_scan_latency_micros(&self) -> Option<u128> {
        (self.scans != 0).then(|| self.total_scan_latency_micros / self.scans as u128)
    }

    pub fn average_event_latency_micros(&self) -> Option<u128> {
        (self.events_emitted != 0)
            .then(|| self.total_event_latency_micros / self.events_emitted as u128)
    }
}

#[derive(Debug, Clone)]
pub struct BoundedEventQueue<T> {
    capacity: usize,
    items: std::collections::VecDeque<T>,
    dropped: u64,
}

impl<T> BoundedEventQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            items: std::collections::VecDeque::with_capacity(capacity),
            dropped: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.capacity == 0 {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        if self.items.len() == self.capacity {
            self.items.pop_front();
            self.dropped = self.dropped.saturating_add(1);
        }
        self.items.push_back(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_is_bounded_and_counts_drops() {
        let mut queue = BoundedEventQueue::new(2);
        queue.push(1);
        queue.push(2);
        queue.push(3);
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.dropped(), 1);
        assert_eq!(queue.pop(), Some(2));
    }
}
