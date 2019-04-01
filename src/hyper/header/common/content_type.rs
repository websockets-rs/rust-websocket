use ::hyper::mime::Mime;

header! {
    /// `Content-Type` header, defined in
    /// [RFC7231](http://tools.ietf.org/html/rfc7231#section-3.1.1.5)
    /// 
    /// The `Content-Type` header field indicates the media type of the
    /// associated representation: either the representation enclosed in the
    /// message payload or the selected representation, as determined by the
    /// message semantics.  The indicated media type defines both the data
    /// format and how that data is intended to be processed by a recipient,
    /// within the scope of the received message semantics, after any content
    /// codings indicated by Content-Encoding are decoded.
    /// 
    /// # ABNF
    /// ```plain
    /// Content-Type = media-type
    /// ```
    /// 
    /// # Example values
    /// * `text/html; charset=ISO-8859-4`
    /// 
    /// # Examples
    /// ```
    /// use hyper::header::{Headers, ContentType};
    /// use hyper::mime::{Mime, TopLevel, SubLevel};
    /// 
    /// let mut headers = Headers::new();
    /// 
    /// headers.set(
    ///     ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![]))
    /// );
    /// ```
    /// ```
    /// use hyper::header::{Headers, ContentType};
    /// use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
    /// 
    /// let mut headers = Headers::new();
    /// 
    /// headers.set(
    ///     ContentType(Mime(TopLevel::Application, SubLevel::Json,
    ///                      vec![(Attr::Charset, Value::Utf8)]))
    /// );
    /// ```
    (ContentType, "Content-Type") => [Mime]
}

impl ContentType {
    /// A constructor  to easily create a `Content-Type: application/json` header.
    #[inline]
    pub(crate) fn json() -> ContentType {
        ContentType(mime!(Application/Json))
    }

    /// A constructor  to easily create a `Content-Type: text/plain; charset=utf-8` header.
    #[inline]
    pub(crate) fn plaintext() -> ContentType {
        ContentType(mime!(Text/Plain; Charset=Utf8))
    }

    /// A constructor  to easily create a `Content-Type: text/html; charset=utf-8` header.
    #[inline]
    pub(crate) fn html() -> ContentType {
        ContentType(mime!(Text/Html; Charset=Utf8))
    }

    /// A constructor  to easily create a `Content-Type: application/www-form-url-encoded` header.
    #[inline]
    pub(crate) fn form_url_encoded() -> ContentType {
        ContentType(mime!(Application/WwwFormUrlEncoded))
    }
    /// A constructor  to easily create a `Content-Type: image/jpeg` header.
    #[inline]
    pub(crate) fn jpeg() -> ContentType {
        ContentType(mime!(Image/Jpeg))
    }

    /// A constructor  to easily create a `Content-Type: image/png` header.
    #[inline]
    pub(crate) fn png() -> ContentType {
        ContentType(mime!(Image/Png))
    }
}

bench_header!(bench, ContentType, { vec![b"application/json; charset=utf-8".to_vec()] });
