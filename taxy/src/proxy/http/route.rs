use super::filter::{FilterResult, RequestFilter};
use hyper::Request;
use taxy_api::site::{Route, SiteEntry};

#[derive(Default, Debug)]
pub struct Router {
    routes: Vec<FilteredRoute>,
}

impl Router {
    pub fn new(entries: Vec<SiteEntry>) -> Self {
        let routes = entries
            .into_iter()
            .flat_map(|entry| {
                entry
                    .site
                    .routes
                    .into_iter()
                    .map(move |route| FilteredRoute {
                        filter: RequestFilter::new(&entry.site.vhosts, &route),
                        route,
                    })
            })
            .collect();
        Self { routes }
    }

    pub fn get_route<T>(&self, req: &Request<T>) -> Option<(&Route, FilterResult)> {
        self.routes
            .iter()
            .find_map(|route| route.filter.test(req).map(|res| (&route.route, res)))
    }
}

#[derive(Debug)]
pub struct FilteredRoute {
    pub filter: RequestFilter,
    pub route: Route,
}
