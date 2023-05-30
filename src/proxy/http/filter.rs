use hyper::Request;

use crate::config::{site::Route, subject_name::SubjectName};

#[derive(Debug, Default)]
pub struct RequestFilter {
    pub vhosts: Vec<SubjectName>,
}

impl RequestFilter {
    pub fn new(vhosts: &[SubjectName], _route: &Route) -> Self {
        Self {
            vhosts: vhosts.to_vec(),
        }
    }

    pub fn test<T>(&self, req: &Request<T>) -> bool {
        let host = req.headers().get("host").and_then(|v| v.to_str().ok());
        match host {
            Some(host) => self.vhosts.iter().any(|vhost| vhost.test(&host)),
            None => false,
        }
    }
}

#[derive(Debug)]
pub struct FilteredRoute {
    pub filter: RequestFilter,
    pub route: Route,
}
