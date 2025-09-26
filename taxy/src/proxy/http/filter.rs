use hyper::Request;
use taxy_api::proxy::Route;
use taxy_api::vhost::VirtualHost;

#[derive(Debug, Default)]
pub struct RequestFilter {
    pub vhosts: Vec<VirtualHost>,
    pub path: Vec<String>,
}

impl RequestFilter {
    pub fn new(vhosts: &[VirtualHost], route: &Route) -> Self {
        Self {
            vhosts: vhosts.to_vec(),
            path: route
                .path
                .split('/')
                .filter(|seg| !seg.is_empty())
                .map(|s| s.to_owned())
                .collect(),
        }
    }

    pub fn test<T>(&self, req: &Request<T>, host: Option<&str>) -> Option<FilterResult> {
        let host_matched = match host {
            Some(host) => self.vhosts.iter().any(|vhost| vhost.test(host)),
            None => false,
        };
        if !host_matched && !self.vhosts.is_empty() {
            return None;
        }
        let path = req.uri().path().trim_start_matches('/').split('/');
        let count = path
            .clone()
            .zip(self.path.iter())
            .take_while(|(a, b)| a == b)
            .count();
        if count == self.path.len() {
            Some(FilterResult {
                path_segments: path.skip(count).map(|s| s.to_string()).collect(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct FilterResult {
    pub path_segments: Vec<String>,
}
