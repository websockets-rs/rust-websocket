//! Utility functions for dealing with URLs

use url::Url;
use hyper::header::Host;

/// Gets a Host header representation from a URL
pub fn url_to_host(url: &Url) -> Option<Host> {
    let host = match url.serialize_host() {
        Some(host) => host,
        None => { return None; }
    };
    let port = match url.port_or_default() {
        Some(port) => port,
        None => { return None; }
    };
    Some(Host {
		hostname: host, 
		port: Some(port)
	})
}