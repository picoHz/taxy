use super::filter::{FilterResult, RequestFilter};
use hyper::header::{HeaderName, HeaderValue, HeaderMap};
use hyper::Request;
use taxy_api::{
    id::ShortId,
    proxy::{ProxyEntry, ProxyKind, Server},
};

fn generate_header_map(headers: &Vec<(String, String)>) -> HeaderMap {
    let mut map = HeaderMap::new();
    for (k, v) in headers {
        if let Ok(key) = HeaderName::from_lowercase(k.to_lowercase().as_bytes()) {
            if let Ok(value) = HeaderValue::from_str(v) {
                map.insert(key, value);
            }
        }
    }
    map
}

#[derive(Default, Debug)]
pub struct Router {
    routes: Vec<FilteredRoute>,
}

impl Router {
    pub fn new(proxies: Vec<ProxyEntry>, https_port: Option<u16>, quic_port: Option<u16>) -> Self {
        let mut routes = vec![];
        for (id, http) in proxies
            .into_iter()
            .filter_map(|entry| match entry.proxy.kind {
                ProxyKind::Http(http) => Some((entry.id, http)),
                _ => None,
            })
        {
            for route in http.routes {
                let filter = RequestFilter::new(&http.vhosts, &route);
                routes.push(FilteredRoute {
                    resource_id: id,
                    filter,
                    route: ParsedRoute {
                        servers: route.servers,
                    },
                    https_port,
                    quic_port,
                    upgrade_insecure: http.upgrade_insecure,
                    custom_headers: generate_header_map(http.custom_headers.as_ref()),
                });
            }
        }
        Self { routes }
    }

    pub fn get_route<T>(
        &self,
        req: &Request<T>,
        host: Option<&str>,
    ) -> Option<(&ParsedRoute, FilterResult, &FilteredRoute)> {
        self.routes.iter().find_map(|route| {
            route
                .filter
                .test(req, host)
                .map(|res| (&route.route, res, route))
        })
    }
}

#[derive(Debug)]
pub struct FilteredRoute {
    pub resource_id: ShortId,
    pub filter: RequestFilter,
    pub route: ParsedRoute,
    pub https_port: Option<u16>,
    pub quic_port: Option<u16>,
    pub upgrade_insecure: bool,
    pub custom_headers: HeaderMap,
}

#[derive(Debug)]
pub struct ParsedRoute {
    pub servers: Vec<Server>,
}
