use super::filter::{FilterResult, RequestFilter};
use hyper::Request;
use taxy_api::{
    id::ShortId,
    proxy::{ProxyEntry, ProxyKind, Server},
};

#[derive(Default, Debug)]
pub struct Router {
    routes: Vec<FilteredRoute>,
}

impl Router {
    pub fn new(proxies: Vec<ProxyEntry>, https_port: Option<u16>) -> Self {
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
                    https_port: https_port.filter(|_| http.upgrade_insecure),
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
}

#[derive(Debug)]
pub struct ParsedRoute {
    pub servers: Vec<Server>,
}
