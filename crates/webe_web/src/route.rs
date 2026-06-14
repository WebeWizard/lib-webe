//! Routing: route patterns, the route table, and deterministic matching.
//!
//! A [`Route`] is a method + path pattern. A [`RouteMap`] holds registered routes
//! and their responders and selects the best match for an incoming request.
//! Selection is deterministic (see [`RouteMap::find_best_route`]): an exact route
//! beats any parameterized route; among parameterized routes the one matching the
//! most path parts wins, ties broken by the earliest wildcard position.

use std::cmp::Ordering::*;
use std::collections::HashMap;

use crate::request::Request;
use crate::responders::Responder;

/// A registered route: an HTTP method paired with a path pattern.
///
/// Path patterns may contain `<name>` parameter segments. A trailing `<name>`
/// segment is *terminal* and captures the remainder of the request path.
#[derive(PartialEq, Eq, Hash)]
pub struct Route {
    /// Uppercased HTTP method (e.g. `GET`).
    pub method: String,
    /// Path pattern, normalized to a leading `/`.
    pub uri: String,
    /// `true` when the pattern contains at least one `<param>` segment.
    pub has_params: bool,
}

impl Route {
    /// Creates a route for `method` and path pattern `uri`.
    ///
    /// The path is normalized to a leading `/` when registered via
    /// [`RouteMap::add_route`], so `"users"` and `"/users"` are equivalent.
    pub fn new(method: &str, uri: &str) -> Route {
        Route {
            method: method.to_owned(),
            uri: uri.to_owned(),
            has_params: uri.contains('<'),
        }
    }
}

/// Why a request could not be routed to a responder.
#[derive(Debug, PartialEq, Eq)]
pub enum RoutingError {
    /// No registered route pattern matched the request path → `404`.
    NotFound,
    /// A route pattern matched the path but no route matched the method → `405`.
    MethodNotAllowed,
}

impl std::fmt::Display for RoutingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingError::NotFound => {
                write!(f, "routing: no route matches the request path (404)")
            }
            RoutingError::MethodNotAllowed => write!(
                f,
                "routing: path matched but the request method is not allowed (405)"
            ),
        }
    }
}

/// The collection of registered routes and their responders.
pub struct RouteMap<'r> {
    inner: HashMap<Route, Box<dyn Responder + 'r>>,
}

impl<'r> Default for RouteMap<'r> {
    fn default() -> Self {
        RouteMap::new()
    }
}

impl<'r> RouteMap<'r> {
    /// Creates an empty route table.
    pub fn new() -> RouteMap<'r> {
        RouteMap {
            inner: HashMap::new(),
        }
    }

    /// Registers `responder` for `route`, normalizing the path to a leading `/`
    /// so leading-slash and no-leading-slash declarations match the same path.
    pub fn add_route<T: 'r + Responder>(&mut self, mut route: Route, responder: T) {
        // normalize: ensure a single leading '/'
        if !route.uri.starts_with('/') {
            route.uri = "/".to_owned() + route.uri.as_str();
        }
        self.inner.insert(route, Box::new(responder));
    }

    /// Returns the responder registered for `route`, if any.
    pub fn responder_for(&self, route: &Route) -> Option<&(dyn Responder + 'r)> {
        self.inner.get(route).map(|boxed| boxed.as_ref())
    }

    /// Selects the best matching route for `request`.
    ///
    /// Selection order (deterministic):
    /// 1. An exact, non-parameterized route matching method + path wins.
    /// 2. Otherwise the most specific parameterized route wins: most matching
    ///    path parts, ties broken by the earliest wildcard position.
    ///
    /// Returns [`RoutingError::MethodNotAllowed`] when the path matches a
    /// registered pattern but no route shares the method, or
    /// [`RoutingError::NotFound`] when no pattern matches the path at all.
    pub fn find_best_route(&self, request: &Request) -> Result<&Route, RoutingError> {
        // 1. exact, non-parameterized match (method + path)
        if let Some(route) = self.inner.keys().find(|route| {
            !route.has_params && request.method == route.method && route.uri == request.uri
        }) {
            return Ok(route);
        }

        let request_parts: Vec<&str> = request.uri.split('/').collect();

        // 2. best path-pattern match among routes that share the method
        let best = self
            .inner
            .keys()
            .filter(|route| route.method == request.method)
            .filter_map(|route| {
                path_match_score(route, &request_parts).map(|(size, wild)| (route, size, wild))
            })
            .max_by(|x, y| match (x.1).cmp(&y.1) {
                Less => Less,
                Greater => Greater,
                Equal => ((x.2).cmp(&y.2)).reverse(),
            });
        if let Some((route, _, _)) = best {
            return Ok(route);
        }

        // 3. no method match. Distinguish 405 (path matched) from 404 (no match).
        let path_matched = self.inner.keys().any(|route| {
            (route.uri == request.uri) || path_match_score(route, &request_parts).is_some()
        });
        if path_matched {
            Err(RoutingError::MethodNotAllowed)
        } else {
            Err(RoutingError::NotFound)
        }
    }
}

/// Scores how well `route`'s path pattern matches the already-split request path.
///
/// Returns `Some((match_size, first_wildcard))` when the pattern matches, where
/// `match_size` is the number of matching leading parts and `first_wildcard` is
/// the 1-based index of the earliest `<param>` part (0 when none). Returns `None`
/// when the route cannot match the path.
fn path_match_score(route: &Route, request_parts: &[&str]) -> Option<(usize, usize)> {
    let route_parts: Vec<&str> = route.uri.split('/').collect();
    // a route with more parts than the request can never match
    if route_parts.len() > request_parts.len() {
        return None;
    }
    let mut match_size = 0;
    let mut first_wild = 0;
    for i in 0..request_parts.len() {
        if request_parts[i] == route_parts[i] || route_parts[i].contains('<') {
            match_size = i + 1;
            if first_wild == 0 && route_parts[i].contains('<') {
                first_wild = i + 1;
            }
            if (i + 1) == route_parts.len() {
                break;
            }
        } else {
            return None; // a non-matching, non-wildcard part rules the route out
        }
    }
    Some((match_size, first_wild))
}

/// Extracts `(name, value)` pairs for a parameterized route matched against a
/// request.
///
/// A non-terminal `<name>` captures exactly one path segment; a terminal
/// `<name>` (the last pattern part) captures the joined remainder of the path.
pub fn parse_route_params(request: &Request, route: &Route) -> Vec<(String, String)> {
    // A request rarely has many params, so a Vec is faster than a small HashMap.
    let mut params: Vec<(String, String)> = Vec::new();

    if !route.has_params {
        return params;
    }

    let request_parts: Vec<&str> = request.uri.split('/').collect();
    let route_uri_parts: Vec<&str> = route.uri.split('/').collect();
    let part_length = route_uri_parts.len();
    for i in 0..part_length {
        if route_uri_parts[i].contains('<') {
            let name = route_uri_parts[i].to_owned();
            let value = if i == part_length - 1 {
                // terminal param: capture the remainder of the request path
                request_parts[i..].join("/")
            } else {
                request_parts[i].to_owned()
            };
            params.push((name, value));
        }
    }
    params
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::responders::Responder;
    use crate::response::Response;
    use crate::validation::Validation;
    use async_trait::async_trait;

    struct Dummy;

    #[async_trait]
    impl Responder for Dummy {
        async fn build_response(
            &self,
            _request: &mut Request,
            _params: &Vec<(String, String)>,
            _validation: Validation,
        ) -> Result<Response, u16> {
            Ok(Response::new(200))
        }
    }

    fn req(method: &str, uri: &str) -> Request<'static> {
        Request {
            total_size: 0,
            method: method.to_owned(),
            uri: uri.to_owned(),
            version: "HTTP/1.1".to_owned(),
            headers: None,
            message_body: None,
        }
    }

    #[test]
    fn distinguishes_404_from_405() {
        let mut map = RouteMap::new();
        map.add_route(Route::new("GET", "/widget"), Dummy);

        // a registered method + path matches
        assert!(map.find_best_route(&req("GET", "/widget")).is_ok());
        // path matches but method does not -> 405
        assert!(matches!(
            map.find_best_route(&req("POST", "/widget")),
            Err(RoutingError::MethodNotAllowed)
        ));
        // no path match at all -> 404
        assert!(matches!(
            map.find_best_route(&req("GET", "/missing")),
            Err(RoutingError::NotFound)
        ));
    }

    #[test]
    fn exact_route_beats_parameterized_route() {
        let mut map = RouteMap::new();
        map.add_route(Route::new("GET", "/files/list"), Dummy);
        map.add_route(Route::new("GET", "/files/<name>"), Dummy);

        let chosen = map.find_best_route(&req("GET", "/files/list")).unwrap();
        assert_eq!(chosen.uri, "/files/list");
        assert!(!chosen.has_params);
    }

    #[test]
    fn terminal_param_captures_remainder() {
        let route = Route::new("GET", "/assets/<path>");
        let params = parse_route_params(&req("GET", "/assets/css/site/main.css"), &route);
        assert_eq!(
            params,
            vec![("<path>".to_owned(), "css/site/main.css".to_owned())]
        );
    }
}
