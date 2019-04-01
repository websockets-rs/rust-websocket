//! Server Requests
//!
//! These are requests that a `hyper::Server` receives, and include its method,
//! target URI, headers, and message body.
use std::io::{self, Read};
use std::net::SocketAddr;
use std::time::Duration;

use ::hyper::buffer::BufReader;
use ::hyper::net::NetworkStream;
use ::hyper::version::{HttpVersion};
use ::hyper::method::Method;
use ::hyper::header::{Headers, ContentLength, TransferEncoding};
use ::hyper::http::h1::{self, Incoming, HttpReader};
use ::hyper::http::h1::HttpReader::{SizedReader, ChunkedReader, EmptyReader};
use ::hyper::uri::RequestUri;

/// A request bundles several parts of an incoming `NetworkStream`, given to a `Handler`.
pub struct Request<'a, 'b: 'a> {
    /// The IP address of the remote connection.
    pub remote_addr: SocketAddr,
    /// The `Method`, such as `Get`, `Post`, etc.
    pub method: Method,
    /// The headers of the incoming request.
    pub headers: Headers,
    /// The target request-uri for this request.
    pub uri: RequestUri,
    /// The version of HTTP for this request.
    pub version: HttpVersion,
    body: HttpReader<&'a mut BufReader<&'b mut NetworkStream>>
}


impl<'a, 'b: 'a> Request<'a, 'b> {
    /// Create a new Request, reading the StartLine and Headers so they are
    /// immediately useful.
    pub(crate) fn new(stream: &'a mut BufReader<&'b mut NetworkStream>, addr: SocketAddr)
        -> ::hyper::Result<Request<'a, 'b>> {

        let Incoming { version, subject: (method, uri), headers } = try!(h1::parse_request(stream));
        debug!("Request Line: {:?} {:?} {:?}", method, uri, version);
        debug!("{:?}", headers);

        let body = if headers.has::<ContentLength>() {
            match headers.get::<ContentLength>() {
                Some(&ContentLength(len)) => SizedReader(stream, len),
                None => unreachable!()
            }
        } else if headers.has::<TransferEncoding>() {
            //todo!("check for Transfer-Encoding: chunked");
            ChunkedReader(stream, None)
        } else {
            EmptyReader(stream)
        };

        Ok(Request {
            remote_addr: addr,
            method: method,
            uri: uri,
            headers: headers,
            version: version,
            body: body
        })
    }

    /// Set the read timeout of the underlying NetworkStream.
    #[inline]
    pub(crate) fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.body.get_ref().get_ref().set_read_timeout(timeout)
    }

    /// Get a reference to the underlying `NetworkStream`.
    #[inline]
    pub(crate) fn downcast_ref<T: NetworkStream>(&self) -> Option<&T> {
        self.body.get_ref().get_ref().downcast_ref()
    }

    /// Get a reference to the underlying Ssl stream, if connected
    /// over HTTPS.
    ///
    /// This is actually just an alias for `downcast_ref`.
    #[inline]
    pub(crate) fn ssl<T: NetworkStream>(&self) -> Option<&T> {
        self.downcast_ref()
    }

    /// Deconstruct a Request into its constituent parts.
    #[inline]
    pub(crate) fn deconstruct(self) -> (SocketAddr, Method, Headers,
                                 RequestUri, HttpVersion,
                                 HttpReader<&'a mut BufReader<&'b mut NetworkStream>>) {
        (self.remote_addr, self.method, self.headers,
         self.uri, self.version, self.body)
    }
}

impl<'a, 'b> Read for Request<'a, 'b> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.body.read(buf)
    }
}

// tests removed
