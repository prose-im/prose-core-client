// prose-core-client/prose-sdk-js
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use alloc::rc::Rc;
use std::io::Write;

use tracing::Level;
use tracing_subscriber::fmt::MakeWriter;

use crate::log::JSLogger;

// tracing requires this, but we're in a single-threaded JS runtime after all…
unsafe impl Send for MakeJSLogWriter {}
unsafe impl Sync for MakeJSLogWriter {}

pub struct MakeJSLogWriter {
    js_logger: Rc<JSLogger>,
}

impl MakeJSLogWriter {
    pub fn new(js_logger: JSLogger) -> Self {
        Self {
            js_logger: Rc::new(js_logger),
        }
    }
}

impl<'a> MakeWriter<'a> for MakeJSLogWriter {
    type Writer = JSLogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        JSLogWriter {
            buffer: vec![],
            level: Level::TRACE,
            panic: false,
            js_logger: self.js_logger.clone(),
        }
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        JSLogWriter {
            buffer: vec![],
            level: *meta.level(),
            panic: meta.fields().field("panic").is_some(),
            js_logger: self.js_logger.clone(),
        }
    }
}

pub struct JSLogWriter {
    buffer: Vec<u8>,
    level: Level,
    panic: bool,
    js_logger: Rc<JSLogger>,
}

impl Write for JSLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for JSLogWriter {
    fn drop(&mut self) {
        let message = String::from_utf8_lossy(&self.buffer);

        match self.level {
            _ if self.level == Level::TRACE => self.js_logger.log_debug(message.as_ref()),
            _ if self.level == Level::DEBUG => self.js_logger.log_debug(message.as_ref()),
            _ if self.level == Level::INFO => self.js_logger.log_info(message.as_ref()),
            _ if self.level == Level::WARN => self.js_logger.log_warn(message.as_ref()),
            _ if self.level == Level::ERROR && self.panic => {
                self.js_logger.log_panic(message.as_ref())
            }
            _ if self.level == Level::ERROR => self.js_logger.log_error(message.as_ref()),
            _ => self.js_logger.log_debug(message.as_ref()),
        }
    }
}
