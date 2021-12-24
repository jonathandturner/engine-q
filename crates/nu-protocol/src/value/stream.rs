use crate::*;
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

/// A single buffer of binary data streamed over multiple parts. Optionally contains ctrl-c that can be used
/// to break the stream.
pub struct ByteStream {
    pub stream: Box<dyn Iterator<Item = Vec<u8>> + Send + 'static>,
    pub ctrlc: Option<Arc<AtomicBool>>,
}
impl ByteStream {
    pub fn into_vec(self) -> Vec<u8> {
        self.flatten().collect::<Vec<u8>>()
    }
}
impl Debug for ByteStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ByteStream").finish()
    }
}

impl Iterator for ByteStream {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ctrlc) = &self.ctrlc {
            if ctrlc.load(Ordering::SeqCst) {
                None
            } else {
                self.stream.next()
            }
        } else {
            self.stream.next()
        }
    }
}

/// A single string streamed over multiple parts. Optionally contains ctrl-c that can be used
/// to break the stream.
pub struct StringStream {
    pub stream: Box<dyn Iterator<Item = String> + Send + 'static>,
    pub ctrlc: Option<Arc<AtomicBool>>,
}
impl StringStream {
    pub fn into_string(self, separator: &str) -> String {
        self.collect::<Vec<String>>().join(separator)
    }

    pub fn from_stream(
        input: impl Iterator<Item = String> + Send + 'static,
        ctrlc: Option<Arc<AtomicBool>>,
    ) -> StringStream {
        StringStream {
            stream: Box::new(input),
            ctrlc,
        }
    }
}
impl Debug for StringStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StringStream").finish()
    }
}

impl Iterator for StringStream {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ctrlc) = &self.ctrlc {
            if ctrlc.load(Ordering::SeqCst) {
                None
            } else {
                self.stream.next()
            }
        } else {
            self.stream.next()
        }
    }
}

/// A potentially infinite stream of values, optinally with a mean to send a Ctrl-C signal to stop
/// the stream from continuing.
///
/// In practice, a "stream" here means anything which can be iterated and produce Values as it iterates.
/// Like other iterators in Rust, observing values from this stream will drain the items as you view them
/// and the stream cannot be replayed.
pub struct ValueStream {
    pub stream: Box<dyn Iterator<Item = Value> + Send + 'static>,
    pub ctrlc: Option<Arc<AtomicBool>>,
}

impl ValueStream {
    pub fn into_string(self, separator: &str, config: &Config) -> String {
        self.map(|x: Value| x.into_string(", ", config))
            .collect::<Vec<String>>()
            .join(separator)
    }

    pub fn from_stream(
        input: impl Iterator<Item = Value> + Send + 'static,
        ctrlc: Option<Arc<AtomicBool>>,
    ) -> ValueStream {
        ValueStream {
            stream: Box::new(input),
            ctrlc,
        }
    }
}

impl Debug for ValueStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValueStream").finish()
    }
}

impl Iterator for ValueStream {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ctrlc) = &self.ctrlc {
            if ctrlc.load(Ordering::SeqCst) {
                None
            } else {
                self.stream.next()
            }
        } else {
            self.stream.next()
        }
    }
}
