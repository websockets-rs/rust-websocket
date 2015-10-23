//! Utility functions for dealing with URLs

use url::Url;
use url::Host as UrlHost;
use hyper::header::Host;
use result::{WebSocketResult, WSUrlErrorKind};

/// Trait that gets required WebSocket URL components
pub trait ToWebSocketUrlComponents {
	/// Retrieve the required WebSocket URL components from this
	fn to_components(&self) -> WebSocketResult<(Host, String, bool)>;
}

impl ToWebSocketUrlComponents for str {
	fn to_components(&self) -> WebSocketResult<(Host, String, bool)> {
		parse_url_str(&self)
	}
}

impl ToWebSocketUrlComponents for Url {
	fn to_components(&self) -> WebSocketResult<(Host, String, bool)> {
		parse_url(&self)
	}
}

impl ToWebSocketUrlComponents for (Host, String, bool) {
	/// Convert a Host, resource name and secure flag to WebSocket URL components.
	fn to_components(&self) -> WebSocketResult<(Host, String, bool)> {
		let (mut host, mut resource_name, secure) = self.clone();
		host.port = Some(match host.port {
			Some(port) => port,
			None => if secure { 443 } else { 80 },
		});
		if resource_name.is_empty() {
			resource_name = "/".to_owned();
		}
		Ok((host, resource_name, secure))
	}
}

impl<'a> ToWebSocketUrlComponents for (Host, &'a str, bool) {
	/// Convert a Host, resource name and secure flag to WebSocket URL components.
	fn to_components(&self) -> WebSocketResult<(Host, String, bool)> {
		(self.0.clone(), self.1.to_owned(), self.2).to_components()
	}
}

impl<'a> ToWebSocketUrlComponents for (Host, &'a str) {
	/// Convert a Host and resource name to WebSocket URL components, assuming an insecure connection.
	fn to_components(&self) -> WebSocketResult<(Host, String, bool)> {
		(self.0.clone(), self.1.to_owned(), false).to_components()
	}
}

impl ToWebSocketUrlComponents for (Host, String) {
	/// Convert a Host and resource name to WebSocket URL components, assuming an insecure connection.
	fn to_components(&self) -> WebSocketResult<(Host, String, bool)> {
		(self.0.clone(), self.1.clone(), false).to_components()
	}
}

impl ToWebSocketUrlComponents for (UrlHost, u16, String, bool) {
	/// Convert a Host, port, resource name and secure flag to WebSocket URL components.
	fn to_components(&self) -> WebSocketResult<(Host, String, bool)> {
		(Host {
			hostname: self.0.serialize(),
			port: Some(self.1)
		}, self.2.clone(), self.3).to_components()
	}
}

impl<'a> ToWebSocketUrlComponents for (UrlHost, u16, &'a str, bool) {
	/// Convert a Host, port, resource name and secure flag to WebSocket URL components.
	fn to_components(&self) -> WebSocketResult<(Host, String, bool)> {
		(Host {
			hostname: self.0.serialize(),
			port: Some(self.1)
		}, self.2, self.3).to_components()
	}
}

impl<'a, T: ToWebSocketUrlComponents> ToWebSocketUrlComponents for &'a T {
	fn to_components(&self) -> WebSocketResult<(Host, String, bool)> {
		(**self).to_components()
	}
}

/// Gets the host, port, resource and secure from the string representation of a url
pub fn parse_url_str(url_str: &str) -> WebSocketResult<(Host, String, bool)> {
    // https://html.spec.whatwg.org/multipage/#parse-a-websocket-url's-components
    // Steps 1 and 2
    let parsed_url = try!(Url::parse(url_str));
    parse_url(&parsed_url)
}

/// Gets the host, port, resource, and secure flag from a url
pub fn parse_url(url: &Url) -> WebSocketResult<(Host, String, bool)> {
    // https://html.spec.whatwg.org/multipage/#parse-a-websocket-url's-components

    // Step 4
    if url.fragment != None {
        return Err(From::from(WSUrlErrorKind::CannotSetFragment));
    }

    let secure = match url.scheme.as_ref() {
        // step 5
        "ws" => false,
        "wss" => true,
        // step 3
        _ => return Err(From::from(WSUrlErrorKind::InvalidScheme)),
    };

    let host = url.host().unwrap().serialize(); // Step 6
    let port = url.port_or_default(); // Steps 7 and 8

    let mut resource = "/".to_owned(); // step 10
    resource.push_str(url.path().unwrap().join("/").as_ref()); // step 9

    // Step 11
    if let Some(ref query) = url.query {
        resource.push('?');
        resource.push_str(query);
    }

    // Step 12
    Ok((Host { hostname: host, port: port }, resource, secure))
}

#[cfg(all(feature = "nightly", test))]
mod tests {
    use super::*;
    //use test;
    use url::{Url, SchemeData, RelativeSchemeData, Host};
    use result::{WebSocketError, WSUrlErrorKind};

    fn url_for_test() -> Url {
        Url {
            fragment: None,
            scheme: "ws".to_owned(),
            scheme_data: SchemeData::Relative(RelativeSchemeData {
                username: "".to_owned(),
                password: None,
                host: Host::Domain("www.example.com".to_owned()),
                port: Some(8080),
                default_port: Some(80),
                path: vec!["some".to_owned(), "path".to_owned()]
            }),
            query: Some("a=b&c=d".to_owned()),
        }
    }

    #[test]
    fn test_parse_url_fragments_not_accepted() {
        let url = &mut url_for_test();
        url.fragment = Some("non_null_fragment".to_owned());

        let result = parse_url(url);
        match result {
            Err(WebSocketError::WebSocketUrlError(
                WSUrlErrorKind::CannotSetFragment)) => (),
            Err(e) => panic!("Expected WSUrlErrorKind::CannotSetFragment but got {}", e),
            Ok(_) => panic!("Expected WSUrlErrorKind::CannotSetFragment but got Ok")
        }
    }

    #[test]
    fn test_parse_url_invalid_schemes_return_error() {
        let url = &mut url_for_test();
        
        let invalid_schemes = &["http", "https", "gopher", "file", "ftp", "other"];
        for scheme in invalid_schemes {
            url.scheme = scheme.to_string();

            let result = parse_url(url);
            match result {
                Err(WebSocketError::WebSocketUrlError(
                    WSUrlErrorKind::InvalidScheme)) => (),
                Err(e) => panic!("Expected WSUrlErrorKind::InvalidScheme but got {}", e),
                Ok(_) => panic!("Expected WSUrlErrorKind::InvalidScheme but got Ok")
            }
        }
    }
    
    #[test]
    fn test_parse_url_valid_schemes_return_ok() {
        let url = &mut url_for_test();
        
        let valid_schemes = &["ws", "wss"];
        for scheme in valid_schemes {
            url.scheme = scheme.to_string();

            let result = parse_url(url);
            match result {
                Ok(_) => (),
                Err(e) => panic!("Expected Ok, but got {}", e)
            }
        }
    }

    #[test]
    fn test_parse_url_ws_returns_unset_secure_flag() {
        let url = &mut url_for_test();
        url.scheme = "ws".to_owned();

        let result = parse_url(url);
        let secure = match result {
            Ok((_, _, secure)) => secure,
            Err(e) => panic!(e),
        };
        assert!(!secure);
    }

    #[test]
    fn test_parse_url_wss_returns_set_secure_flag() {
        let url = &mut url_for_test();
        url.scheme = "wss".to_owned();

        let result = parse_url(url);
        let secure = match result {
            Ok((_, _, secure)) => secure,
            Err(e) => panic!(e),
        };
        assert!(secure);
    }
    
    #[test]
    fn test_parse_url_generates_proper_output() {
        let url = &url_for_test();

        let result = parse_url(url);
        let (host, resource) = match result {
            Ok((host, resource, _)) => (host, resource),
            Err(e) => panic!(e),
        };
        
        assert_eq!(host.hostname, "www.example.com".to_owned());
        assert_eq!(resource, "/some/path?a=b&c=d".to_owned());

        match host.port {
            Some(port) => assert_eq!(port, 8080),
            _ => panic!("Port should not be None"),
        }
    }

    #[test]
    fn test_parse_url_empty_path_should_give_slash() {
        let url = &mut url_for_test();
        match url.scheme_data {
            SchemeData::Relative(ref mut scheme_data) => { scheme_data.path = vec![]; },
            _ => ()
        }

        let result = parse_url(url);
        let resource = match result {
            Ok((_, resource, _)) => resource,
            Err(e) => panic!(e),
        };

        assert_eq!(resource, "/?a=b&c=d".to_owned());
    }

    #[test]
    fn test_parse_url_none_query_should_not_append_question_mark() {
        let url = &mut url_for_test();
        url.query = None;

        let result = parse_url(url);
        let resource = match result {
            Ok((_, resource, _)) => resource,
            Err(e) => panic!(e),
        };

        assert_eq!(resource, "/some/path".to_owned());
    }

    #[test]
    fn test_parse_url_none_port_should_use_default_port() {
        let url = &mut url_for_test();
        match url.scheme_data {
            SchemeData::Relative(ref mut scheme_data) => {
                scheme_data.port = None;
            },
            _ => ()
        }

        let result = parse_url(url);
        let host = match result {
            Ok((host, _, _)) => host,
            Err(e) => panic!(e),
        };

        match host.port {
            Some(80) => (),
            Some(p) => panic!("Expected port to be 80 but got {}", p),
            None => panic!("Expected port to be 80 but got `None`"),
        }
    }

    #[test]
    fn test_parse_url_str_valid_url1() {
        let url_str = "ws://www.example.com/some/path?a=b&c=d";
        let result = parse_url_str(url_str);
        let (host, resource, secure) = match result {
            Ok((host, resource, secure)) => (host, resource, secure),
            Err(e) => panic!(e),
        };

        match host.port {
            Some(80) => (),
            Some(p) => panic!("Expected port 80 but got {}", p),
            None => panic!("Expected port 80 but got `None`")
        }
        assert_eq!(host.hostname, "www.example.com".to_owned());
        assert_eq!(resource, "/some/path?a=b&c=d".to_owned());
        assert!(!secure);
    }

    #[test]
    fn test_parse_url_str_valid_url2() {
        let url_str = "wss://www.example.com";
        let result = parse_url_str(url_str);
        let (host, resource, secure) = match result {
            Ok((host, resource, secure)) => (host, resource, secure),
            Err(e) => panic!(e)
        };

        match host.port {
            Some(443) => (),
            Some(p) => panic!("Expected port 443 but got {}", p),
            None => panic!("Expected port 443 but got `None`")
        }
        assert_eq!(host.hostname, "www.example.com".to_owned());
        assert_eq!(resource, "/".to_owned());
        assert!(secure);
    }

    #[test]
    fn test_parse_url_str_invalid_relative_url() {
        let url_str = "/some/relative/path?a=b&c=d";
        let result = parse_url_str(url_str);
        match result {
            Err(WebSocketError::UrlError(_)) => (),
            Err(e) => panic!("Expected UrlError, but got unexpected error {}", e),
            Ok(_) => panic!("Expected UrlError, but got Ok"),
        }
    }

    #[test]
    fn test_parse_url_str_invalid_url_scheme() {
        let url_str = "http://www.example.com/some/path?a=b&c=d";
        let result = parse_url_str(url_str);
        match result {
            Err(WebSocketError::WebSocketUrlError(WSUrlErrorKind::InvalidScheme)) => (),
            Err(e) => panic!("Expected InvalidScheme, but got unexpected error {}", e),
            Ok(_) => panic!("Expected InvalidScheme, but got Ok"),
        }
    }

    #[test]
    fn test_parse_url_str_invalid_url_fragment() {
        let url_str = "http://www.example.com/some/path#some-id";
        let result = parse_url_str(url_str);
        match result {
            Err(WebSocketError::WebSocketUrlError(WSUrlErrorKind::CannotSetFragment)) => (),
            Err(e) => panic!("Expected CannotSetFragment, but got unexpected error {}", e),
            Ok(_) => panic!("Expected CannotSetFragment, but got Ok"),
        }
    }
}
