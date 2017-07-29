//! Types to represent HTTP responses.
use std::fmt;

use header::{Header, Headers};
use http::{MessageHead, Body};
use status::StatusCode;
use version::HttpVersion;

/// A response message head.
pub type ResponseHead = MessageHead<StatusCode>;

/// An HTTP Response
pub struct Response<B = Body> {
    version: HttpVersion,
    headers: Headers,
    status: StatusCode,
    body: Option<B>,
}

impl<B> Response<B> {
    /// Constructs a default response
    #[inline]
    pub fn new() -> Response<B> {
        Response::default()
    }

    /// Get the HTTP version of this response.
    #[inline]
    pub fn version(&self) -> HttpVersion { self.version }

    /// Get the headers from the response.
    #[inline]
    pub fn headers(&self) -> &Headers { &self.headers }

    /// Get a mutable reference to the headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut Headers { &mut self.headers }

    /// Get the status from the server.
    #[inline]
    pub fn status(&self) -> StatusCode { self.status }

    /// Set the `StatusCode` for this response.
    #[inline]
    pub fn set_status(&mut self, status: StatusCode) {
        self.status = status;
    }

    /// Set the status and move the Response.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.set_status(status);
        self
    }

    /// Set a header and move the Response.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Set the headers and move the Response.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    /// Set the body.
    #[inline]
    pub fn set_body<T: Into<B>>(&mut self, body: T) {
        self.body = Some(body.into());
    }

    /// Set the body and move the Response.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_body<T: Into<B>>(mut self, body: T) -> Self {
        self.set_body(body);
        self
    }

    /// Read the body.
    #[inline]
    pub fn body_ref(&self) -> Option<&B> { self.body.as_ref() }

    /// Constructs a response using a ResponseHead and optional body.
    #[inline]
    pub fn pack(head: ResponseHead, body: Option<B>) -> Response<B> {
        info!("Response::new \"{} {}\"", head.version, head.subject);
        debug!("Response::new headers={:?}", head.headers);

        Response::<B> {
            status: head.subject,
            version: head.version,
            headers: head.headers,
            body: body,
        }
    }

    /// Deconstructs a response into a ResponseHead and optional bodyu.
    #[inline]
    pub fn unpack(self) -> (MessageHead<StatusCode>, Option<B>) {
        let head = MessageHead::<StatusCode> {
            version: self.version,
            headers: self.headers,
            subject: self.status,
        };
        (head, self.body)
    }
}

impl Response<Body> {
    /// Take the `Body` of this response.
    #[inline]
    pub fn body(self) -> Body {
        self.body.unwrap_or_default()
    }
}

impl<B> Default for Response<B> {
    fn default() -> Response<B> {
        Response::<B> {
            version: Default::default(),
            headers: Default::default(),
            status: Default::default(),
            body: None,
        }
    }
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Response")
            .field("status", &self.status)
            .field("version", &self.version)
            .field("headers", &self.headers)
            .finish()
    }
}
