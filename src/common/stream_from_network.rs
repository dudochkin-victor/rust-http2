#![allow(dead_code)]

use std::sync::mpsc;

use futures::Poll;
use futures::stream::Stream;

use stream_part::*;

use futures_misc::signal;
use futures_misc::ResultOrEof;

use error;


pub struct StreamFromNetwork {
    signal: signal::Receiver,
    rx: mpsc::Receiver<ResultOrEof<HttpStreamPart, error::Error>>,
}

impl Stream for StreamFromNetwork {
    type Item = HttpStreamPart;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.rx.try_recv() {
                Ok(_v) => {

                }
                Err(mpsc::TryRecvError::Empty) => {
                    unimplemented!()
                },
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(error::Error::Other("channel disconnected, likely conn died"));
                },
            }
        }
    }
}