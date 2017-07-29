//! Pieces pertaining to the HTTP message protocol.
use std::fmt;

use bytes::BytesMut;

use header::{Connection, ConnectionOption, Expect};
use header::Headers;
use method::Method;
use version::HttpVersion;
use version::HttpVersion::{Http10, Http11};

pub use self::conn::{Conn, KeepAlive, KA};
pub use self::body::{Body, TokioBody};
pub use self::chunk::Chunk;
pub use self::request::{RequestHead, RequestLine};
pub use self::response::ResponseHead;

mod body;
mod chunk;
mod conn;
mod io;
mod h1;
//mod h2;
pub mod request;
pub mod response;


/// Head of an HTTP message. Includes version, request or status line, and
/// headers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MessageHead<S> {
    /// HTTP version of the message.
    pub version: HttpVersion,
    /// Subject (request line or status line) of message.
    pub subject: S,
    /// Headers of the message.
    pub headers: Headers
}

impl<S> MessageHead<S> {
    /// Creates a new MessageHead with the given HTTP version and subject and
    /// empty headers.
    pub fn new(version: HttpVersion, subject: S) -> Self {
        MessageHead {
            version: version,
            subject: subject,
            headers: Headers::new(),
        }
    }
}

/// Checks if a connection should be kept alive.
#[inline]
pub fn should_keep_alive<S>(head: &MessageHead<S>) -> bool {
    let ret = match (head.version, head.headers.get::<Connection>()) {
        (Http10, None) => false,
        (Http10, Some(conn)) if !conn.contains(&ConnectionOption::KeepAlive) => false,
        (Http11, Some(conn)) if conn.contains(&ConnectionOption::Close)  => false,
        _ => true
    };
    trace!("should_keep_alive(version={:?}, header={:?}) = {:?}", head.version, head.headers.get::<Connection>(), ret);
    ret
}

/// Checks if a connection is expecting a `100 Continue` before sending its body.
#[inline]
pub fn expecting_continue<S>(head: &MessageHead<S>) -> bool {
    let ret = match (head.version, head.headers.get::<Expect>()) {
        (Http11, Some(&Expect::Continue)) => true,
        _ => false
    };
    trace!("expecting_continue(version={:?}, header={:?}) = {:?}", head.version, head.headers.get::<Expect>(), ret);
    ret
}

#[derive(Debug)]
pub enum ServerTransaction {}

#[derive(Debug)]
pub enum ClientTransaction {}

pub trait Http1Transaction {
    type Incoming;
    type Outgoing: Default;
    fn parse(bytes: &mut BytesMut) -> ParseResult<Self::Incoming>;
    fn decoder(head: &MessageHead<Self::Incoming>, method: &mut Option<::Method>) -> ::Result<h1::Decoder>;
    fn encode(head: MessageHead<Self::Outgoing>, has_body: bool, method: &mut Option<Method>, dst: &mut Vec<u8>) -> h1::Encoder;
}

pub type ParseResult<T> = ::Result<Option<(MessageHead<T>, usize)>>;

struct DebugTruncate<'a>(&'a [u8]);

impl<'a> fmt::Debug for DebugTruncate<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.0;
        if bytes.len() > 32 {
            try!(f.write_str("["));
            for byte in &bytes[..32] {
                try!(write!(f, "{:?}, ", byte));
            }
            write!(f, "... {}]", bytes.len())
        } else {
            fmt::Debug::fmt(bytes, f)
        }
    }
}

#[test]
fn test_should_keep_alive() {
    let mut http10 = MessageHead {
        version: Http10,
        subject: (),
        headers: Headers::new(),
    };
    let mut http11 = MessageHead {
        version: Http11,
        subject: (),
        headers: Headers::new(),
    };

    assert!(!should_keep_alive(&http10));
    assert!(should_keep_alive(&http11));

    http10.headers.set(Connection::close());
    http11.headers.set(Connection::close());
    assert!(!should_keep_alive(&http10));
    assert!(!should_keep_alive(&http11));

    http10.headers.set(Connection::keep_alive());
    http11.headers.set(Connection::keep_alive());
    assert!(should_keep_alive(&http10));
    assert!(should_keep_alive(&http11));
}

#[test]
fn test_expecting_continue() {
    let mut http10 = MessageHead {
        version: Http10,
        subject: (),
        headers: Headers::new(),
    };
    let mut http11 = MessageHead {
        version: Http11,
        subject: (),
        headers: Headers::new(),
    };

    assert!(!expecting_continue(&http10));
    assert!(!expecting_continue(&http11));

    http10.headers.set(Expect::Continue);
    http11.headers.set(Expect::Continue);
    assert!(!expecting_continue(&http10));
    assert!(expecting_continue(&http11));
}
