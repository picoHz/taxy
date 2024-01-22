use super::filter::{FilterResult, RequestFilter};
use hyper::Request;
use taxy_api::{
    error::Error,
    id::ShortId,
    proxy::{ProxyEntry, ProxyKind, Route, Server},
};
use tokio_rustls::rustls::pki_types::ServerName;
use url::Url;
use warp::host::Authority;

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
                    resource_id: id,
                    filter: RequestFilter::new(&http.vhosts, &route),
                    route: route.try_into().unwrap(),
                })
            })
            .collect();
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
    pub path: String,
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
        Ok(Self {
            path: route.path,
            servers,
        })
    }
}

#[derive(Debug)]
pub struct ParsedServer {
    pub url: Url,
    pub authority: Authority,
    pub server_name: ServerName<'static>,
}

impl TryFrom<Server> for ParsedServer {
    type Error = Error;

    fn try_from(server: Server) -> Result<Self, Self::Error> {
        let hostname = server
            .url
            .host_str()
            .ok_or_else(|| Error::InvalidServerUrl {
                url: server.url.clone(),
            })?;
        let authority = format!(
            "{}:{}",
            hostname,
            server.url.port_or_known_default().unwrap_or_default()
        )
        .parse()
        .map_err(|_| Error::InvalidServerUrl {
            url: server.url.clone(),
        })?;
        let server_name = ServerName::try_from(hostname)
            .map_err(|_| Error::InvalidServerUrl {
                url: server.url.clone(),
            })?
            .to_owned();
        Ok(Self {
            url: server.url,
            authority,
            server_name,
        })
    }
}
