use taxy_api::site::ProxyEntry;

pub struct HealthChecker {}

impl HealthChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, entries: &[ProxyEntry]) {
        println!("{:?}", entries);
    }

    pub async fn start_checks(&mut self) {
        println!("HealthChecker::update");
    }
}
