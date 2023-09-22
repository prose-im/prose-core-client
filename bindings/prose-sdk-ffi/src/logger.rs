// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Mutex;
use std::time::Instant;

use tracing::field::{Field, Visit};
use tracing::span::Attributes;
use tracing::{Event, Id, Level, Subscriber};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[derive(Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

pub trait Logger: Send {
    fn log(&self, level: LogLevel, message: String);
}

pub fn set_logger(logger: Box<dyn Logger>, max_level: LogLevel) {
    tracing_subscriber::registry()
        .with(CustomLogger::new(logger).with_filter(LevelFilter::from_level(max_level.into())))
        .init();
}

impl From<&Level> for LogLevel {
    fn from(value: &Level) -> Self {
        match value {
            &Level::TRACE => LogLevel::Trace,
            &Level::DEBUG => LogLevel::Debug,
            &Level::INFO => LogLevel::Info,
            &Level::WARN => LogLevel::Warn,
            &Level::ERROR => LogLevel::Error,
        }
    }
}

impl From<LogLevel> for Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

pub struct CustomLogger {
    logger: Mutex<Box<dyn Logger>>,
}

impl CustomLogger {
    fn new(logger: Box<dyn Logger>) -> Self {
        CustomLogger {
            logger: Mutex::new(logger),
        }
    }
}

impl<S> Layer<S> for CustomLogger
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, _attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        // let span = ctx.span(id).expect("Encountered invalid span");
        //
        // let metadata = span.metadata();
        // let mut attributes = AttributeMap::default();
        // let mut attr_visitor = FieldVisitor::new(&mut attributes);
        // attrs.record(&mut attr_visitor);
        //
        // let name = {
        //     let function_name = [metadata.target(), metadata.name()].join("::");
        //     let full_name = format!(
        //         "{}({})",
        //         function_name,
        //         attributes
        //             .into_iter()
        //             .map(|(k, v)| format!("{}: {}", k, v))
        //             .collect::<Vec<_>>()
        //             .join(", ")
        //     );
        //     println!("+++ {}", full_name);
        // };
        //
        // let mut extensions = span.extensions_mut();
        // extensions.insert(Activity::new());
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let mut attributes = AttributeMap::default();
        let mut attr_visitor = FieldVisitor::new(&mut attributes);
        event.record(&mut attr_visitor);

        let mut message = String::new();
        if let Some(value) = attributes.remove(&"message".to_string()) {
            message = value;
            message.push_str("  ");
        }
        message.push_str(
            &attributes
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" "),
        );

        self.logger
            .lock()
            .unwrap()
            .log(metadata.level().into(), message);
    }

    fn on_enter(&self, _id: &Id, _ctx: Context<'_, S>) {
        // let span = ctx.span(id).expect("Encountered invalid span");
        // let metadata = span.metadata();
        // let function_name = [metadata.target(), metadata.name()].join("::");
        // println!("+++ On enter: {}", function_name);
        //
        // let mut extensions = span.extensions_mut();
        // let mut activity = extensions
        //     .get_mut::<Activity>()
        //     .expect("Span didn't contain Activity");
        // activity.start_time = Instant::now();
    }

    fn on_exit(&self, _id: &Id, _ctx: Context<'_, S>) {
        // let span = ctx.span(id).expect("Encountered invalid span");
        // let metadata = span.metadata();
        // let function_name = [metadata.target(), metadata.name()].join("::");
        //
        // let extensions = span.extensions();
        // let activity = extensions
        //     .get::<Activity>()
        //     .expect("Span didn't contain Activity");
        // println!(
        //     "+++ On exit: {} after {:.2?}",
        //     function_name,
        //     activity.start_time.elapsed()
        // );
    }

    fn on_close(&self, _id: Id, _ctx: Context<'_, S>) {
        // let span = ctx.span(&id).expect("Encountered invalid span");
        // let metadata = span.metadata();
        // let function_name = [metadata.target(), metadata.name()].join("::");
        // println!("+++ On close: {}", function_name);
        //
        // let mut extensions = span.extensions_mut();
        // extensions
        //     .remove::<Activity>()
        //     .expect("Span didn't contain Activity");
    }
}

#[allow(dead_code)]
struct Activity {
    start_time: Instant,
}

#[allow(dead_code)]
impl Activity {
    fn new() -> Self {
        Activity {
            start_time: Instant::now(),
        }
    }
}

type AttributeMap = BTreeMap<String, String>;

struct FieldVisitor<'a> {
    output: &'a mut AttributeMap,
}

impl<'a> FieldVisitor<'a> {
    fn new(output: &'a mut AttributeMap) -> Self {
        FieldVisitor { output }
    }
}

impl<'a> Visit for FieldVisitor<'a> {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.output
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.output
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.output
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.output
            .insert(field.name().to_string(), format!("\"{}\"", value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.output
            .insert(field.name().to_string(), format!("{:?}", value));
    }
}
