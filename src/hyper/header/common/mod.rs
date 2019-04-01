//! A Collection of Header implementations for common HTTP Headers.
//!
//! ## Mime
//!
//! Several header fields use MIME values for their contents. Keeping with the
//! strongly-typed theme, the [mime](http://seanmonstar.github.io/mime.rs) crate
//! is used, such as `ContentType(pub(crate) Mime)`.

pub(crate) use self::accept::Accept;
pub(crate) use self::access_control_allow_credentials::AccessControlAllowCredentials;
pub(crate) use self::access_control_allow_headers::AccessControlAllowHeaders;
pub(crate) use self::access_control_allow_methods::AccessControlAllowMethods;
pub(crate) use self::access_control_allow_origin::AccessControlAllowOrigin;
pub(crate) use self::access_control_expose_headers::AccessControlExposeHeaders;
pub(crate) use self::access_control_max_age::AccessControlMaxAge;
pub(crate) use self::access_control_request_headers::AccessControlRequestHeaders;
pub(crate) use self::access_control_request_method::AccessControlRequestMethod;
pub(crate) use self::accept_charset::AcceptCharset;
pub(crate) use self::accept_encoding::AcceptEncoding;
pub(crate) use self::accept_language::AcceptLanguage;
pub(crate) use self::accept_ranges::{AcceptRanges, RangeUnit};
pub(crate) use self::allow::Allow;
pub(crate) use self::authorization::{Authorization, Scheme, Basic, Bearer};
pub(crate) use self::cache_control::{CacheControl, CacheDirective};
pub(crate) use self::connection::{Connection, ConnectionOption};
pub(crate) use self::content_disposition::{ContentDisposition, DispositionType, DispositionParam};
pub(crate) use self::content_length::ContentLength;
pub(crate) use self::content_encoding::ContentEncoding;
pub(crate) use self::content_language::ContentLanguage;
pub(crate) use self::content_range::{ContentRange, ContentRangeSpec};
pub(crate) use self::content_type::ContentType;
pub(crate) use self::cookie::Cookie;
pub(crate) use self::date::Date;
pub(crate) use self::etag::ETag;
pub(crate) use self::expect::Expect;
pub(crate) use self::expires::Expires;
pub(crate) use self::from::From;
pub(crate) use self::host::Host;
pub(crate) use self::if_match::IfMatch;
pub(crate) use self::if_modified_since::IfModifiedSince;
pub(crate) use self::if_none_match::IfNoneMatch;
pub(crate) use self::if_unmodified_since::IfUnmodifiedSince;
pub(crate) use self::if_range::IfRange;
pub(crate) use self::last_modified::LastModified;
pub(crate) use self::location::Location;
pub(crate) use self::origin::Origin;
pub(crate) use self::pragma::Pragma;
pub(crate) use self::prefer::{Prefer, Preference};
pub(crate) use self::preference_applied::PreferenceApplied;
pub(crate) use self::range::{Range, ByteRangeSpec};
pub(crate) use self::referer::Referer;
pub(crate) use self::referrer_policy::ReferrerPolicy;
pub(crate) use self::server::Server;
pub(crate) use self::set_cookie::SetCookie;
pub(crate) use self::strict_transport_security::StrictTransportSecurity;
pub(crate) use self::transfer_encoding::TransferEncoding;
pub(crate) use self::upgrade::{Upgrade, Protocol, ProtocolName};
pub(crate) use self::user_agent::UserAgent;
pub(crate) use self::vary::Vary;
pub(crate) use self::link::{Link, LinkValue, RelationType, MediaDesc};

#[doc(hidden)]
#[macro_export]
macro_rules! bench_header(
    ($name:ident, $ty:ty, $value:expr) => {
        #[cfg(test)]
        #[cfg(feature = "nightly")]
        #[allow(deprecated)]
        mod $name {
            use test::Bencher;
            use super::*;

            use ::hyper::header::{Header, HeaderFormatter};

            #[bench]
            fn bench_parse(b: &mut Bencher) {
                let val = $value;
                b.iter(|| {
                    let _: $ty = Header::parse_header(&val[..]).unwrap();
                });
            }

            #[bench]
            fn bench_format(b: &mut Bencher) {
                let val: $ty = Header::parse_header(&$value[..]).unwrap();
                let fmt = HeaderFormatter(&val);
                b.iter(|| {
                    format!("{}", fmt);
                });
            }
        }
    }
);

#[doc(hidden)]
#[macro_export]
macro_rules! __hyper__deref {
    ($from:ty => $to:ty) => {
        impl ::std::ops::Deref for $from {
            type Target = $to;

            fn deref(&self) -> &$to {
                &self.0
            }
        }

        impl ::std::ops::DerefMut for $from {
            fn deref_mut(&mut self) -> &mut $to {
                &mut self.0
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __hyper__tm {
    ($id:ident, $tm:ident{$($tf:item)*}) => {
        #[allow(unused_imports)]
        #[cfg(test)]
        mod $tm{
            use std::str;
            use $crate::hyper::header::*;
            use $crate::hyper::mime::*;
            use $crate::hyper::language_tags::*;
            use $crate::hyper::method::Method;
            use super::$id as HeaderField;
            $($tf)*
        }

    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_header {
    ($id:ident, $raw:expr) => {
        #[test]
        fn $id() {
            #[allow(unused_imports)]
            use std::ascii::AsciiExt;
            let raw = $raw;
            let a: Vec<Vec<u8>> = raw.iter().map(|x| x.to_vec()).collect();
            let value = HeaderField::parse_header(&a[..]);
            let result = format!("{}", value.unwrap());
            let expected = String::from_utf8(raw[0].to_vec()).unwrap();
            let result_cmp: Vec<String> = result
                .to_ascii_lowercase()
                .split(' ')
                .map(|x| x.to_owned())
                .collect();
            let expected_cmp: Vec<String> = expected
                .to_ascii_lowercase()
                .split(' ')
                .map(|x| x.to_owned())
                .collect();
            assert_eq!(result_cmp.concat(), expected_cmp.concat());
        }
    };
    ($id:ident, $raw:expr, $typed:expr) => {
        #[test]
        fn $id() {
            let a: Vec<Vec<u8>> = $raw.iter().map(|x| x.to_vec()).collect();
            let val = HeaderField::parse_header(&a[..]);
            let typed: Option<HeaderField> = $typed;
            // Test parsing
            assert_eq!(val.ok(), typed);
            // Test formatting
            if typed.is_some() {
                let raw = &($raw)[..];
                let mut iter = raw.iter().map(|b|str::from_utf8(&b[..]).unwrap());
                let mut joined = String::new();
                joined.push_str(iter.next().unwrap());
                for s in iter {
                    joined.push_str(", ");
                    joined.push_str(s);
                }
                assert_eq!(format!("{}", typed.unwrap()), joined);
            }
        }
    }
}

#[macro_export]
macro_rules! header {
    // $a:meta: Attributes associated with the header item (usually docs)
    // $id:ident: Identifier of the header
    // $n:expr: Lowercase name of the header
    // $nn:expr: Nice name of the header

    // List header, zero or more items
    ($(#[$a:meta])*($id:ident, $n:expr) => ($item:ty)*) => {
        $(#[$a])*
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $id(pub(crate) Vec<$item>);
        __hyper__deref!($id => Vec<$item>);
        impl $crate::hyper::header::Header for $id {
            fn header_name() -> &'static str {
                $n
            }
            fn parse_header(raw: &[Vec<u8>]) -> $crate::hyper::Result<Self> {
                $crate::hyper::header::parsing::from_comma_delimited(raw).map($id)
            }
        }
        impl $crate::hyper::header::HeaderFormat for $id {
            fn fmt_header(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                $crate::hyper::header::parsing::fmt_comma_delimited(f, &self.0[..])
            }
        }
        impl ::std::fmt::Display for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                use $crate::hyper::header::HeaderFormat;
                self.fmt_header(f)
            }
        }
    };
    // List header, one or more items
    ($(#[$a:meta])*($id:ident, $n:expr) => ($item:ty)+) => {
        $(#[$a])*
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $id(pub(crate) Vec<$item>);
        __hyper__deref!($id => Vec<$item>);
        impl $crate::hyper::header::Header for $id {
            fn header_name() -> &'static str {
                $n
            }
            fn parse_header(raw: &[Vec<u8>]) -> $crate::hyper::Result<Self> {
                $crate::hyper::header::parsing::from_comma_delimited(raw).map($id)
            }
        }
        impl $crate::hyper::header::HeaderFormat for $id {
            fn fmt_header(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                $crate::hyper::header::parsing::fmt_comma_delimited(f, &self.0[..])
            }
        }
        impl ::std::fmt::Display for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                use $crate::hyper::header::HeaderFormat;
                self.fmt_header(f)
            }
        }
    };
    // Single value header
    ($(#[$a:meta])*($id:ident, $n:expr) => [$value:ty]) => {
        $(#[$a])*
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $id(pub(crate) $value);
        __hyper__deref!($id => $value);
        impl $crate::hyper::header::Header for $id {
            fn header_name() -> &'static str {
                $n
            }
            fn parse_header(raw: &[Vec<u8>]) -> $crate::hyper::Result<Self> {
                $crate::hyper::header::parsing::from_one_raw_str(raw).map($id)
            }
        }
        impl $crate::hyper::header::HeaderFormat for $id {
            fn fmt_header(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&**self, f)
            }
        }
        impl ::std::fmt::Display for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&**self, f)
            }
        }
    };
    // List header, one or more items with "*" option
    ($(#[$a:meta])*($id:ident, $n:expr) => {Any / ($item:ty)+}) => {
        $(#[$a])*
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) enum $id {
            /// Any value is a match
            Any,
            /// Only the listed items are a match
            Items(Vec<$item>),
        }
        impl $crate::hyper::header::Header for $id {
            fn header_name() -> &'static str {
                $n
            }
            fn parse_header(raw: &[Vec<u8>]) -> $crate::hyper::Result<Self> {
                // FIXME: Return None if no item is in $id::Only
                if raw.len() == 1 {
                    if raw[0] == b"*" {
                        return Ok($id::Any)
                    }
                }
                $crate::hyper::header::parsing::from_comma_delimited(raw).map($id::Items)
            }
        }
        impl $crate::hyper::header::HeaderFormat for $id {
            fn fmt_header(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match *self {
                    $id::Any => f.write_str("*"),
                    $id::Items(ref fields) => $crate::hyper::header::parsing::fmt_comma_delimited(
                        f, &fields[..])
                }
            }
        }
        impl ::std::fmt::Display for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                use $crate::hyper::header::HeaderFormat;
                self.fmt_header(f)
            }
        }
    };

    // optional test module
    ($(#[$a:meta])*($id:ident, $n:expr) => ($item:ty)* $tm:ident{$($tf:item)*}) => {
        header! {
            $(#[$a])*
            ($id, $n) => ($item)*
        }

        __hyper__tm! { $id, $tm { $($tf)* }}
    };
    ($(#[$a:meta])*($id:ident, $n:expr) => ($item:ty)+ $tm:ident{$($tf:item)*}) => {
        header! {
            $(#[$a])*
            ($id, $n) => ($item)+
        }

        __hyper__tm! { $id, $tm { $($tf)* }}
    };
    ($(#[$a:meta])*($id:ident, $n:expr) => [$item:ty] $tm:ident{$($tf:item)*}) => {
        header! {
            $(#[$a])*
            ($id, $n) => [$item]
        }

        __hyper__tm! { $id, $tm { $($tf)* }}
    };
    ($(#[$a:meta])*($id:ident, $n:expr) => {Any / ($item:ty)+} $tm:ident{$($tf:item)*}) => {
        header! {
            $(#[$a])*
            ($id, $n) => {Any / ($item)+}
        }

        __hyper__tm! { $id, $tm { $($tf)* }}
    };
}


mod accept;
mod access_control_allow_credentials;
mod access_control_allow_headers;
mod access_control_allow_methods;
mod access_control_allow_origin;
mod access_control_expose_headers;
mod access_control_max_age;
mod access_control_request_headers;
mod access_control_request_method;
mod accept_charset;
mod accept_encoding;
mod accept_language;
mod accept_ranges;
mod allow;
mod authorization;
mod cache_control;
mod cookie;
mod connection;
mod content_disposition;
mod content_encoding;
mod content_language;
mod content_length;
mod content_range;
mod content_type;
mod date;
mod etag;
mod expect;
mod expires;
mod from;
mod host;
mod if_match;
mod if_modified_since;
mod if_none_match;
mod if_range;
mod if_unmodified_since;
mod last_modified;
mod location;
mod origin;
mod pragma;
mod prefer;
mod preference_applied;
mod range;
mod referer;
mod referrer_policy;
mod server;
mod set_cookie;
mod strict_transport_security;
mod transfer_encoding;
mod upgrade;
mod user_agent;
mod vary;
mod link;
