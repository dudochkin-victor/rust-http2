use futures::future;
use futures::future::Future;
use futures::stream;
use futures::stream::Stream;

use bytes::Bytes;

use solicit_async::*;
use solicit::header::Headers;
use http_common::*;
use message::SimpleHttpMessage;

use solicit::HttpError;

/// Convenient wrapper around async HTTP response future/stream
pub struct HttpResponse(pub HttpFutureSend<(Headers, HttpPartStream)>);

impl HttpResponse {
    // constructors

    pub fn new<F>(future: F) -> HttpResponse
        where F : Future<Item=(Headers, HttpPartStream), Error=HttpError> + Send + 'static
    {
        HttpResponse(Box::new(future))
    }

    pub fn headers_and_stream(headers: Headers, stream: HttpPartStream) -> HttpResponse
    {
        HttpResponse::new(future::ok((headers, stream)))
    }

    pub fn headers_and_bytes_stream<S>(headers: Headers, content: S) -> HttpResponse
        where S : Stream<Item=Bytes, Error=HttpError> + Send + 'static
    {
        HttpResponse::headers_and_stream(headers, HttpPartStream::bytes(content))
    }

    pub fn headers_and_bytes(header: Headers, content: Bytes) -> HttpResponse {
        HttpResponse::headers_and_bytes_stream(header, stream::once(Ok(content)))
    }

    pub fn message(message: SimpleHttpMessage) -> HttpResponse {
        HttpResponse::headers_and_bytes(message.headers, message.body)
    }

    pub fn from_stream<S>(stream: S) -> HttpResponse
        where S : Stream<Item=HttpStreamPart, Error=HttpError> + Send + 'static
    {
        HttpResponse::new(stream.into_future().map_err(|(p, _s)| p).and_then(|(first, rem)| {
            match first {
                Some(part) => {
                    match part.content {
                        HttpStreamPartContent::Headers(headers) => {
                            Ok((headers, HttpPartStream::new(rem)))
                        },
                        HttpStreamPartContent::Data(..) => {
                            Err(HttpError::InvalidFrame("data before headers".to_owned()))
                        }
                    }
                }
                None => {
                    Err(HttpError::InvalidFrame("empty response, expecting headers".to_owned()))
                }
            }
        }))
    }

    pub fn err(err: HttpError) -> HttpResponse {
        HttpResponse::new(future::err(err))
    }

    // getters

    pub fn into_stream_flag(self) -> HttpFutureStreamSend<HttpStreamPart> {
        Box::new(self.0.map(|(headers, rem)| {
            // NOTE: flag may be wrong for first item
            stream::once(Ok(HttpStreamPart::intermediate_headers(headers))).chain(rem)
        }).flatten_stream())
    }

    pub fn into_stream(self) -> HttpFutureStreamSend<HttpStreamPartContent> {
        Box::new(self.into_stream_flag().map(|c| c.content))
    }

    pub fn collect(self) -> HttpFutureSend<SimpleHttpMessage> {
        Box::new(self.into_stream().fold(SimpleHttpMessage::new(), |mut c, p| {
            c.add(p);
            Ok::<_, HttpError>(c)
        }))
    }
}
