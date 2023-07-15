use taxy_api::{
    health_check::{HealthCheck, HealthCheckResult},
    site::ProxyEntry,
};

#[derive(Debug)]
pub struct HealthChecker {
    entries: Vec<HealthCheckEntry>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn update(&mut self, entries: &[ProxyEntry]) {
        self.entries = entries
            .iter()
            .filter_map(|entry| entry.proxy.health_check.clone())
            .map(HealthCheckEntry::new)
            .collect();
        println!("HealthChecker::update {:?}", self.entries);
    }

    pub async fn start_checks(&mut self) {
        println!("HealthChecker::update");
    }
}

#[derive(Debug)]
struct HealthCheckEntry {
    check: HealthCheck,
    history: Vec<HealthCheckResult>,
}

impl HealthCheckEntry {
    fn new(check: HealthCheck) -> Self {
        Self {
            check,
            history: Vec::new(),
        }
    }
}
