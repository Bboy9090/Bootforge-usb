//! Device Caching Layer - OMEGA MODE
//!
//! Provides caching for device information to reduce USB bus queries.

use crate::model::UsbDeviceRecord;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cached device entry
#[derive(Debug, Clone)]
pub struct CachedDevice {
    /// The device record
    pub device: UsbDeviceRecord,
    /// When the entry was cached
    pub cached_at: Instant,
    /// Number of cache hits
    pub hits: u64,
    /// Last access time
    pub last_access: Instant,
}

impl CachedDevice {
    /// Create a new cached entry
    pub fn new(device: UsbDeviceRecord) -> Self {
        let now = Instant::now();
        Self {
            device,
            cached_at: now,
            hits: 0,
            last_access: now,
        }
    }
    
    /// Check if the entry is expired
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() > ttl
    }
    
    /// Get age of the cache entry
    pub fn age(&self) -> Duration {
        self.cached_at.elapsed()
    }
    
    /// Record a cache hit
    pub fn hit(&mut self) {
        self.hits += 1;
        self.last_access = Instant::now();
    }
}

/// Cache key for device lookup
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CacheKey {
    /// Key by VID:PID
    VidPid(u16, u16),
    /// Key by bus and address
    BusAddress(u8, u8),
    /// Key by serial number
    Serial(String),
    /// Key by port path
    PortPath(String),
    /// Composite key
    Composite(String),
}

impl CacheKey {
    /// Create a composite key from a device
    pub fn from_device(device: &UsbDeviceRecord) -> Self {
        // Prefer serial number
        if let Some(ref serial) = device.descriptor.serial_number {
            return Self::Serial(serial.clone());
        }
        
        // Then port path
        if let Some(ref path) = device.location.port_path {
            return Self::PortPath(path.clone());
        }
        
        // Then bus/address
        if let (Some(bus), Some(addr)) = (device.location.bus, device.location.address) {
            return Self::BusAddress(bus, addr);
        }
        
        // Fallback to VID:PID (not unique but better than nothing)
        Self::VidPid(device.id.vid, device.id.pid)
    }
}

/// Type alias for VID:PID index
type VidPidIndex = HashMap<(u16, u16), Vec<CacheKey>>;

/// Device cache with configurable TTL and eviction
pub struct DeviceCache {
    /// Cached entries by key
    entries: Arc<RwLock<HashMap<CacheKey, CachedDevice>>>,
    /// Cache by VID:PID for quick lookups (may have multiple devices)
    by_vid_pid: Arc<RwLock<VidPidIndex>>,
    /// Time-to-live for cache entries
    ttl: Duration,
    /// Maximum cache size
    max_size: usize,
}

impl DeviceCache {
    /// Create a new cache with default settings
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            by_vid_pid: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(30),
            max_size: 1000,
        }
    }
    
    /// Create a cache with custom TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }
    
    /// Create a cache with custom max size
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }
    
    /// Get a device from the cache
    pub fn get(&self, key: &CacheKey) -> Option<UsbDeviceRecord> {
        let mut entries = self.entries.write().unwrap();
        
        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired(self.ttl) {
                entries.remove(key);
                return None;
            }
            entry.hit();
            return Some(entry.device.clone());
        }
        
        None
    }
    
    /// Get a device by VID:PID (returns first match)
    pub fn get_by_vid_pid(&self, vid: u16, pid: u16) -> Option<UsbDeviceRecord> {
        let by_vid_pid = self.by_vid_pid.read().unwrap();
        
        if let Some(keys) = by_vid_pid.get(&(vid, pid)) {
            for key in keys {
                if let Some(device) = self.get(key) {
                    return Some(device);
                }
            }
        }
        
        None
    }
    
    /// Get all devices matching VID:PID
    pub fn get_all_by_vid_pid(&self, vid: u16, pid: u16) -> Vec<UsbDeviceRecord> {
        let by_vid_pid = self.by_vid_pid.read().unwrap();
        let mut results = Vec::new();
        
        if let Some(keys) = by_vid_pid.get(&(vid, pid)) {
            for key in keys {
                if let Some(device) = self.get(key) {
                    results.push(device);
                }
            }
        }
        
        results
    }
    
    /// Insert a device into the cache
    pub fn insert(&self, device: UsbDeviceRecord) {
        let key = CacheKey::from_device(&device);
        let vid_pid = (device.id.vid, device.id.pid);
        
        // Evict if necessary
        self.maybe_evict();
        
        // Insert into main cache
        {
            let mut entries = self.entries.write().unwrap();
            entries.insert(key.clone(), CachedDevice::new(device));
        }
        
        // Update VID:PID index
        {
            let mut by_vid_pid = self.by_vid_pid.write().unwrap();
            by_vid_pid.entry(vid_pid).or_default().push(key);
        }
    }
    
    /// Insert multiple devices
    pub fn insert_all(&self, devices: impl IntoIterator<Item = UsbDeviceRecord>) {
        for device in devices {
            self.insert(device);
        }
    }
    
    /// Remove a device from the cache
    pub fn remove(&self, key: &CacheKey) -> Option<UsbDeviceRecord> {
        let mut entries = self.entries.write().unwrap();
        entries.remove(key).map(|e| e.device)
    }
    
    /// Clear expired entries
    pub fn clear_expired(&self) {
        let mut entries = self.entries.write().unwrap();
        let expired: Vec<CacheKey> = entries.iter()
            .filter(|(_, v)| v.is_expired(self.ttl))
            .map(|(k, _)| k.clone())
            .collect();
        
        for key in expired {
            entries.remove(&key);
        }
    }
    
    /// Clear all entries
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
        
        let mut by_vid_pid = self.by_vid_pid.write().unwrap();
        by_vid_pid.clear();
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let entries = self.entries.read().unwrap();
        
        let total_hits: u64 = entries.values().map(|e| e.hits).sum();
        let expired_count = entries.values().filter(|e| e.is_expired(self.ttl)).count();
        
        CacheStats {
            size: entries.len(),
            max_size: self.max_size,
            total_hits,
            expired_count,
            ttl: self.ttl,
        }
    }
    
    /// Evict entries if cache is full
    fn maybe_evict(&self) {
        let entries = self.entries.read().unwrap();
        if entries.len() < self.max_size {
            return;
        }
        drop(entries);
        
        // Remove expired first
        self.clear_expired();
        
        // If still too full, remove least recently used
        let entries = self.entries.read().unwrap();
        if entries.len() >= self.max_size {
            drop(entries);
            
            let mut entries = self.entries.write().unwrap();
            
            // Find LRU entry
            if let Some((lru_key, _)) = entries.iter()
                .min_by_key(|(_, v)| v.last_access)
                .map(|(k, v)| (k.clone(), v.clone()))
            {
                entries.remove(&lru_key);
            }
        }
    }
    
    /// Get all cached devices
    pub fn all(&self) -> Vec<UsbDeviceRecord> {
        let entries = self.entries.read().unwrap();
        entries.values()
            .filter(|e| !e.is_expired(self.ttl))
            .map(|e| e.device.clone())
            .collect()
    }
    
    /// Check if a device is cached
    pub fn contains(&self, key: &CacheKey) -> bool {
        let entries = self.entries.read().unwrap();
        if let Some(entry) = entries.get(key) {
            !entry.is_expired(self.ttl)
        } else {
            false
        }
    }
    
    /// Number of entries in the cache
    pub fn len(&self) -> usize {
        let entries = self.entries.read().unwrap();
        entries.len()
    }
    
    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for DeviceCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current cache size
    pub size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Total cache hits
    pub total_hits: u64,
    /// Number of expired entries
    pub expired_count: usize,
    /// Cache TTL
    pub ttl: Duration,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DeviceCache: {}/{} entries, {} hits, {} expired, TTL {:?}",
            self.size, self.max_size, self.total_hits, self.expired_count, self.ttl
        )
    }
}

/// Cached device enumeration wrapper
/// 
/// Note: This provides caching for UsbDeviceRecord. Convert from UsbDeviceInfo
/// using the provided conversion methods if needed.
pub struct CachedEnumerator {
    cache: Arc<DeviceCache>,
}

impl CachedEnumerator {
    /// Create a new cached enumerator
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DeviceCache::new()),
        }
    }
    
    /// Create with custom cache
    pub fn with_cache(cache: Arc<DeviceCache>) -> Self {
        Self { cache }
    }
    
    /// Get all cached devices
    pub fn get_cached(&self) -> Vec<UsbDeviceRecord> {
        self.cache.all()
    }
    
    /// Insert devices into the cache
    pub fn cache_devices(&self, devices: impl IntoIterator<Item = UsbDeviceRecord>) {
        self.cache.insert_all(devices);
    }
    
    /// Clear the cache
    pub fn clear(&self) {
        self.cache.clear();
    }
    
    /// Get cache reference
    pub fn cache(&self) -> &DeviceCache {
        &self.cache
    }
}

impl Default for CachedEnumerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn make_device(vid: u16, pid: u16, serial: &str) -> UsbDeviceRecord {
        UsbDeviceRecord {
            id: UsbId::new(vid, pid),
            location: UsbLocation {
                bus: Some(1),
                address: Some(1),
                port_path: None,
            },
            descriptor: UsbDescriptorSummary {
                manufacturer: None,
                product: None,
                serial_number: Some(serial.to_string()),
                device_class: None,
                device_subclass: None,
                device_protocol: None,
                usb_version: None,
            },
            driver: DriverStatus::Unknown,
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        }
    }

    #[test]
    fn test_cache_insert_get() {
        let cache = DeviceCache::new();
        let device = make_device(0x1234, 0x5678, "ABC123");
        
        cache.insert(device.clone());
        
        let key = CacheKey::Serial("ABC123".to_string());
        let cached = cache.get(&key);
        
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().id.vid, 0x1234);
    }

    #[test]
    fn test_cache_vid_pid() {
        let cache = DeviceCache::new();
        let device = make_device(0x1234, 0x5678, "DEF456");
        
        cache.insert(device);
        
        let result = cache.get_by_vid_pid(0x1234, 0x5678);
        assert!(result.is_some());
    }

    #[test]
    fn test_cache_stats() {
        let cache = DeviceCache::new();
        cache.insert(make_device(0x1111, 0x2222, "SN1"));
        cache.insert(make_device(0x3333, 0x4444, "SN2"));
        
        let stats = cache.stats();
        assert_eq!(stats.size, 2);
    }

    #[test]
    fn test_cache_clear() {
        let cache = DeviceCache::new();
        cache.insert(make_device(0x1234, 0x5678, "SN"));
        assert!(!cache.is_empty());
        
        cache.clear();
        assert!(cache.is_empty());
    }
}
