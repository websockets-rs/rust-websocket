#![allow(warnings)]

pub(crate) extern crate httparse;
pub(crate) extern crate mime;
pub(crate) extern crate language_tags;
pub(crate) extern crate traitobject;
pub(crate) extern crate typeable;
pub(crate) extern crate time;
pub(crate) extern crate url;

pub(crate) mod http;
pub(crate) mod buffer;
pub(crate) mod error;
pub(crate) mod method;
pub(crate) mod header;
pub(crate) mod status;
pub(crate) mod version;
pub(crate) mod uri;
pub(crate) mod net;
pub(crate) mod server;

pub(crate) use self::error::{Error,Result};
pub(crate) use self::server::Server;
