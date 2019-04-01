use ::hyper::mime::Mime;

use ::hyper::header::{QualityItem, qitem};

header! {
    /// `Accept` header, defined in [RFC7231](http://tools.ietf.org/html/rfc7231#section-5.3.2)
    ///
    /// The `Accept` header field can be used by user agents to specify
    /// response media types that are acceptable.  Accept header fields can
    /// be used to indicate that the request is specifically limited to a
    /// small set of desired types, as in the case of a request for an
    /// in-line image
    ///
    /// # ABNF
    /// ```plain
    /// Accept = #( media-range [ accept-params ] )
    ///
    /// media-range    = ( "*/*"
    ///                  / ( type "/" "*" )
    ///                  / ( type "/" subtype )
    ///                  ) *( OWS ";" OWS parameter )
    /// accept-params  = weight *( accept-ext )
    /// accept-ext = OWS ";" OWS token [ "=" ( token / quoted-string ) ]
    /// ```
    ///
    /// # Example values
    /// * `audio/*; q=0.2, audio/basic` (`*` value won't parse correctly)
    /// * `text/plain; q=0.5, text/html, text/x-dvi; q=0.8, text/x-c`
    ///
    /// # Examples
    /// ```
    /// use hyper::header::{Headers, Accept, qitem};
    /// use hyper::mime::{Mime, TopLevel, SubLevel};
    ///
    /// let mut headers = Headers::new();
    ///
    /// headers.set(
    ///     Accept(vec![
    ///         qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
    ///     ])
    /// );
    /// ```
    /// ```
    /// use hyper::header::{Headers, Accept, qitem};
    /// use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
    ///
    /// let mut headers = Headers::new();
    /// headers.set(
    ///     Accept(vec![
    ///         qitem(Mime(TopLevel::Application, SubLevel::Json,
    ///                    vec![(Attr::Charset, Value::Utf8)])),
    ///     ])
    /// );
    /// ```
    /// ```
    /// use hyper::header::{Headers, Accept, QualityItem, Quality, qitem};
    /// use hyper::mime::{Mime, TopLevel, SubLevel};
    ///
    /// let mut headers = Headers::new();
    ///
    /// headers.set(
    ///     Accept(vec![
    ///         qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
    ///         qitem(Mime(TopLevel::Application,
    ///                    SubLevel::Ext("xhtml+xml".to_owned()), vec![])),
    ///         QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]),
    ///                          Quality(900)),
    ///                          qitem(Mime(TopLevel::Image,
    ///                                     SubLevel::Ext("webp".to_owned()), vec![])),
    ///                          QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]),
    ///                                           Quality(800))
    ///     ])
    /// );
    /// ```
    ///
    /// # Notes
    /// * Using always Mime types to represent `media-range` differs from the ABNF.
    /// * **FIXME**: `accept-ext` is not supported.
    (Accept, "Accept") => (QualityItem<Mime>)+
}

impl Accept {
    /// A constructor to easily create `Accept: */*`.
    pub(crate) fn star() -> Accept {
        Accept(vec![qitem(mime!(Star/Star))])
    }

    /// A constructor to easily create `Accept: application/json`.
    pub(crate) fn json() -> Accept {
        Accept(vec![qitem(mime!(Application/Json))])
    }

    /// A constructor to easily create `Accept: text/*`.
    pub(crate) fn text() -> Accept {
        Accept(vec![qitem(mime!(Text/Star))])
    }

    /// A constructor to easily create `Accept: image/*`.
    pub(crate) fn image() -> Accept {
        Accept(vec![qitem(mime!(Image/Star))])
    }
}


bench_header!(bench, Accept, { vec![b"text/plain; q=0.5, text/html".to_vec()] });
