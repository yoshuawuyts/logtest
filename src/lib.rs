//! Test and assert log statements
//!
//! This crate makes it easy to test and assert [log](https://docs.rs/log)
//! messages. Logging can be a crucial part of any API, and ensuring log
//! messages are tested is an important in preventing accidental regressions.
//!
//! # Constraints
//!
//! `logtest` uses a per-binary message queue to store messages in. This
//! enables it to work in any async setting, and captures the global ordering of
//! messages for the code being tested.
//! Because [Rust spawns per-test threads during integration
//! testing](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html),
//! when testing log statements it's recommended to only have a single
//! `#[test]` block per test file. That prevents possible race conditions from
//! tests from running in parallel.
//!
//! A convention we recommend is adding a `test/log.rs` file that contains a
//! single `#[test]` block that drives all log assertions. Splitting the code
//! can be done by calling out to regular fuctions from the `#[test]` function.
//!
//! # Examples
//!
//! ```
//! use logtest::Logger;
//!
//! // Start the logger.
//! let mut logger = Logger::start();
//!
//! // Log some messages.
//! log::info!("hello");
//! log::info!("world");
//!
//! // The messages are now available from the logger.
//! assert_eq!(logger.pop().unwrap().args(), "hello");
//! assert_eq!(logger.pop().unwrap().args(), "world");
//! ```

#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

use lazy_static::lazy_static;
use log::{kv, Level, LevelFilter, Metadata};
use std::collections::{HashMap, VecDeque};
use std::iter::Iterator;
use std::sync::Mutex;

/// The "payload" of a log message.
#[derive(Debug, PartialEq, Eq)]
pub struct Record {
    args: String,
    level: Level,
    target: String,
    key_values: HashMap<String, String>,
}

impl Record {
    /// The message body.
    pub fn args(&self) -> &str {
        &self.args
    }

    ///The verbosity level of the message.
    pub fn level(&self) -> Level {
        self.level
    }

    /// The name of the target of the directive.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// The structured key-value pairs associated with the message.
    pub fn key_values(&self) -> Vec<(String, String)> {
        self.key_values
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect()
    }
}

lazy_static! {
    /// The internal queue of events.
    static ref EVENTS: Mutex<VecDeque<Record>> = Mutex::new(VecDeque::new());
}

/// A log key-value visitor.
struct Visitor {
    pairs: HashMap<String, String>,
}

impl<'kvs> kv::Visitor<'kvs> for Visitor {
    fn visit_pair(&mut self, key: kv::Key<'kvs>, val: kv::Value<'kvs>) -> Result<(), kv::Error> {
        self.pairs.insert(format!("{}", key), val.to_string());
        Ok(())
    }
}

/// The logger impl. This is for internal use only.
#[derive(Debug)]
struct LoggerInternal;

impl log::Log for LoggerInternal {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let mut visitor = Visitor {
                pairs: HashMap::new(),
            };
            record
                .key_values()
                .visit(&mut visitor)
                .expect("could not visit kv pairs");
            EVENTS.lock().unwrap().push_back(Record {
                args: format!("{}", record.args()),
                level: record.level(),
                target: record.target().to_owned(),
                key_values: visitor.pairs,
            });
        }
    }
    fn flush(&self) {}
}

/// The test logger.
#[derive(Debug)]
pub struct Logger;

impl Logger {
    /// Create a new instance of `Logger` and start listening for events.
    pub fn start() -> Self {
        log::set_logger(&LoggerInternal).unwrap();
        log::set_max_level(LevelFilter::Trace);
        Self {}
    }

    /// Pop an event from the front of the event queue.
    #[must_use]
    pub fn pop(&mut self) -> Option<Record> {
        EVENTS.lock().unwrap().pop_front()
    }

    /// Returns the number of elements in the `Logger`.
    pub fn len(&mut self) -> usize {
        EVENTS.lock().unwrap().len()
    }

    /// Returns `true` if the `Logger` is empty.
    pub fn is_empty(&mut self) -> bool {
        EVENTS.lock().unwrap().is_empty()
    }
}

/// Create a new instance of `Logger` and start listening for events.
pub fn start() -> Logger {
    Logger::start()
}

impl Iterator for Logger {
    type Item = Record;
    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}
