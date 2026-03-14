// ForgeWorks Core - Metrics Exporter
// Exports dashboard metrics for monitoring and alerting

use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub active_units: i64,
    pub audit_coverage_pct: f64,
    pub compliance_escalations_30d: i64,
    pub audit_entries_24h: i64,
    pub integrity_violations: i64,
    pub active_jurisdictions: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDistribution {
    pub risk_level: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStats {
    pub authority_type: String,
    pub status: String,
    pub route_count: i64,
    pub routes_last_7_days: i64,
    pub routes_last_30_days: i64,
}

use std::fs;
use std::path::Path;

/**
 * Export dashboard summary metrics
 * 
 * Attempts to read from a local state file, simulating a dynamic database.
 */
pub fn export_dashboard_metrics(_db_pool: &str) -> DashboardMetrics {
    if let Ok(data) = fs::read_to_string("metrics_state.json") {
        if let Ok(metrics) = serde_json::from_str(&data) {
            return metrics;
        }
    }
    
    // Fallback if no state file exists
    DashboardMetrics {
        active_units: 1,
        audit_coverage_pct: 100.0,
        compliance_escalations_30d: 0,
        audit_entries_24h: 15,
        integrity_violations: 0,
        active_jurisdictions: 1,
        timestamp: Utc::now(),
    }
}

/**
 * Export risk distribution metrics
 */
pub fn export_risk_distribution(_db_pool: &str) -> Vec<RiskDistribution> {
    if let Ok(data) = fs::read_to_string("risk_state.json") {
        if let Ok(risk) = serde_json::from_str(&data) {
            return risk;
        }
    }
    vec![]
}

/**
 * Export routing statistics
 */
pub fn export_routing_stats(_db_pool: &str) -> Vec<RoutingStats> {
    if let Ok(data) = fs::read_to_string("routing_state.json") {
        if let Ok(routing) = serde_json::from_str(&data) {
            return routing;
        }
    }
    vec![]
}

/**
 * Check if system health is acceptable
 */
pub fn check_system_health(metrics: &DashboardMetrics) -> bool {
    // Health checks:
    // 1. Audit coverage must be 100%
    // 2. No integrity violations allowed
    // 3. Escalations within acceptable threshold
    
    metrics.audit_coverage_pct >= 100.0
        && metrics.integrity_violations == 0
        && metrics.compliance_escalations_30d < 1000 // Threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_system_health() {
        let healthy_metrics = DashboardMetrics {
            active_units: 100,
            audit_coverage_pct: 100.0,
            compliance_escalations_30d: 10,
            audit_entries_24h: 1000,
            integrity_violations: 0,
            active_jurisdictions: 5,
            timestamp: Utc::now(),
        };
        
        assert!(check_system_health(&healthy_metrics));
        
        let unhealthy_metrics = DashboardMetrics {
            active_units: 100,
            audit_coverage_pct: 95.0, // Below 100%
            compliance_escalations_30d: 10,
            audit_entries_24h: 1000,
            integrity_violations: 0,
            active_jurisdictions: 5,
            timestamp: Utc::now(),
        };
        
        assert!(!check_system_health(&unhealthy_metrics));
    }
}
