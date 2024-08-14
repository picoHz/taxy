use super::filter::{FilterResult, RequestFilter};
use hyper::Request;
use std::str::FromStr;
use taxy_api::{
    error::Error,
    id::ShortId,
    proxy::{ProxyEntry, ProxyKind, Route, Server},
};
use tracing::error;
use url::Url;
use warp::host::Authority;

#[derive(Default, Debug)]
pub struct Router {
    routes: Vec<FilteredRoute>,
}

impl Router {
    pub fn new(entries: Vec<ProxyEntry>) -> Self {
        let mut routes = vec![];
        for (id, http) in entries
            .into_iter()
            .filter_map(|entry| match entry.proxy.kind {
                ProxyKind::Http(http) => Some((entry.id, http)),
                _ => None,
            })
        {
            for route in http.routes {
                let filter = RequestFilter::new(&http.vhosts, &route);
                match route.try_into() {
                    Ok(route) => {
                        routes.push(FilteredRoute {
                            resource_id: id,
                            filter,
                            route,
                        });
                    }
                    Err(e) => {
                        error!("Failed to parse route: {:?}", e);
                    }
                }
            }
        }
        Self { routes }
    }

    pub fn get_route<T>(&self, req: &Request<T>) -> Option<(&ParsedRoute, FilterResult, ShortId)> {
        self.routes.iter().find_map(|route| {
            route
                .filter
                .test(req)
                .map(|res| (&route.route, res, route.resource_id))
        })
    }
}

#[derive(Debug)]
pub struct FilteredRoute {
    pub resource_id: ShortId,
    pub filter: RequestFilter,
    pub route: ParsedRoute,
}

#[derive(Debug)]
pub struct ParsedRoute {
    pub servers: Vec<ParsedServer>,
}

impl TryFrom<Route> for ParsedRoute {
    type Error = Error;

    fn try_from(route: Route) -> Result<Self, Self::Error> {
        let servers = route
            .servers
            .into_iter()
            .map(ParsedServer::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { servers })
    }
}

#[derive(Debug)]
pub struct ParsedServer {
    pub url: Url,
    pub authority: Authority,
}

impl TryFrom<Server> for ParsedServer {
    type Error = Error;

    fn try_from(server: Server) -> Result<Self, Self::Error> {
        let url = server.url.clone().into();
        let authority = server.url.authority().ok_or(Error::InvalidServerUrl {
            url: server.url.to_string(),
        })?;
        let hostname = server.url.hostname().ok_or(Error::InvalidServerUrl {
            url: server.url.to_string(),
        })?;
        Ok(Self {
            url,
            authority: Authority::from_str(&authority).map_err(|_| Error::InvalidServerUrl {
                url: hostname.to_owned(),
            })?,
        })
    }
}
