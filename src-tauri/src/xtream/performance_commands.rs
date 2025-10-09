use crate::xtream::performance_monitor::{PerformanceMetrics, PerformanceMonitor};
use std::sync::Arc;
use tauri::State;

/// State for performance monitoring
pub struct PerformanceState {
    pub monitor: Arc<PerformanceMonitor>,
}

impl PerformanceState {
    pub fn new(monitor: Arc<PerformanceMonitor>) -> Self {
        Self { monitor }
    }
}

/// Get current performance metrics
#[tauri::command]
pub fn get_performance_metrics(
    state: State<'_, PerformanceState>,
) -> Result<PerformanceMetrics, String> {
    Ok(state.monitor.get_metrics())
}

/// Reset performance metrics
#[tauri::command]
pub fn reset_performance_metrics(
    state: State<'_, PerformanceState>,
) -> Result<(), String> {
    state.monitor.reset_metrics();
    Ok(())
}

/// Get cache hit ratio
#[tauri::command]
pub fn get_cache_hit_ratio(
    state: State<'_, PerformanceState>,
) -> Result<f64, String> {
    let metrics = state.monitor.get_metrics();
    let total = metrics.cache_metrics.total_hits + metrics.cache_metrics.total_misses;
    if total == 0 {
        Ok(0.0)
    } else {
        Ok(metrics.cache_metrics.hit_rate)
    }
}

/// Get API success rate
#[tauri::command]
pub fn get_api_success_rate(
    state: State<'_, PerformanceState>,
) -> Result<f64, String> {
    let metrics = state.monitor.get_metrics();
    let total = metrics.api_metrics.total_requests;
    if total == 0 {
        Ok(0.0)
    } else {
        Ok(metrics.api_metrics.successful_requests as f64 / total as f64)
    }
}

/// Get slow operations (operations with avg duration > threshold_ms)
#[tauri::command]
pub fn get_slow_operations(
    state: State<'_, PerformanceState>,
    threshold_ms: u64,
) -> Result<Vec<String>, String> {
    use std::time::Duration;
    
    let threshold = Duration::from_millis(threshold_ms);
    let slow_ops = state.monitor.get_slow_operations(threshold);
    
    Ok(slow_ops.iter().map(|op| op.operation_name.clone()).collect())
}
