use std::borrow::Cow;

use ::hyper::header::Connection;
use ::hyper::header::ConnectionOption::{KeepAlive, Close};
use ::hyper::header::Headers;
use ::hyper::version::HttpVersion;
use ::hyper::version::HttpVersion::{Http10, Http11};


pub(crate) use self::message::{HttpMessage, RequestHead, ResponseHead, Protocol};

pub(crate) mod h1;
pub(crate) mod message;

/// The raw status code and reason-phrase.
#[derive(Clone, PartialEq, Debug)]
pub struct RawStatus(pub u16, pub Cow<'static, str>);

/// Checks if a connection should be kept alive.
#[inline]
pub(crate) fn should_keep_alive(version: HttpVersion, headers: &Headers) -> bool {
    trace!("should_keep_alive( {:?}, {:?} )", version, headers.get::<Connection>());
    match (version, headers.get::<Connection>()) {
        (Http10, None) => false,
        (Http10, Some(conn)) if !conn.contains(&KeepAlive) => false,
        (Http11, Some(conn)) if conn.contains(&Close)  => false,
        _ => true
    }
}
