use hyper::header::HOST;
use hyper::http::uri::Authority;
use hyper::{Request, Uri};
use std::str::FromStr;
use taxy_api::proxy::Route;
use taxy_api::subject_name::SubjectName;

#[derive(Debug, Default)]
pub struct RequestFilter {
    pub vhosts: Vec<SubjectName>,
    pub path: Vec<String>,
}

impl RequestFilter {
    pub fn new(vhosts: &[SubjectName], route: &Route) -> Self {
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

    pub fn test<T>(&self, req: &Request<T>) -> Option<FilterResult> {
        let auth = req.uri().authority();
        let header_auth = if auth.is_none() {
            req.headers()
                .get(HOST)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| Authority::from_str(s).ok())
        } else {
            None
        };
        let host = auth.or(header_auth.as_ref()).map(|a| a.host());

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
            let new_path = "/".to_string() + &path.skip(count).collect::<Vec<_>>().join("/");
            FilterResult::new(&new_path).ok()
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct FilterResult {
    pub uri: Uri,
}

impl FilterResult {
    pub fn new(new_path: &str) -> anyhow::Result<Self> {
        let uri = Uri::from_str(new_path)?;
        Ok(Self { uri })
    }
}
