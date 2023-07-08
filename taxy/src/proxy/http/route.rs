use super::filter::{FilterResult, RequestFilter};
use hyper::Request;
use taxy_api::site::{ProxyEntry, ProxyKind, Route};

#[derive(Default, Debug)]
pub struct Router {
    routes: Vec<FilteredRoute>,
}

impl Router {
    pub fn new(entries: Vec<ProxyEntry>) -> Self {
        let routes = entries
            .into_iter()
            .filter_map(|entry| match entry.proxy.kind {
                ProxyKind::Http(http) => Some((entry.id, http)),
                _ => None,
            })
            .flat_map(|(id, http)| {
                http.routes.into_iter().map(move |route| FilteredRoute {
                    resource_id: id.clone(),
                    filter: RequestFilter::new(&http.vhosts, &route),
                    route,
                })
            })
            .collect();
        Self { routes }
    }

    pub fn get_route<T>(&self, req: &Request<T>) -> Option<(&Route, FilterResult, &str)> {
        self.routes.iter().find_map(|route| {
            route
                .filter
                .test(req)
                .map(|res| (&route.route, res, route.resource_id.as_str()))
        })
    }
}

#[derive(Debug)]
pub struct FilteredRoute {
    pub resource_id: String,
    pub filter: RequestFilter,
    pub route: Route,
}
