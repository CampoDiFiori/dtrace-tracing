mod bindings;

use std::ffi::CString;

use tracing::{field::Visit, info, instrument, span, Level};
use tracing_subscriber::{filter, prelude::__tracing_subscriber_SubscriberExt, Layer};

use crate::bindings::*;

pub struct U;

impl Default for U {
    fn default() -> Self {
        Self
    }
}

struct RecordVisitor {
    message: String,
    fields: serde_json::Map<String, serde_json::Value>,
}

impl Visit for RecordVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }

        self.fields.insert(
            field.name().to_owned(),
            serde_json::Value::String(format!("{value:?}")),
        );
    }
}

impl RecordVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: Default::default(),
        }
    }

    fn fields(&self) -> String {
        serde_json::to_string(&self.fields).unwrap()
    }

    fn message(self) -> String {
        self.message
    }
}

impl<S> tracing_subscriber::Layer<S> for U
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut rv = RecordVisitor::new();
        event.record(&mut rv);

        unsafe {
            let c = CString::new(rv.fields()).unwrap();
            tracing_event(c.into_raw());
        }
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        _id: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut rv = RecordVisitor::new();
        attrs.record(&mut rv);
        unsafe {
            let c = CString::new(rv.fields()).unwrap();
            tracing_enter(c.into_raw());
        }
    }
}

#[instrument]
pub fn main() {
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter::LevelFilter::WARN))
        .with(U);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let mut i = 0;
    loop {
        if i % 2 == 0 {
            even(i, i % 3 == 0);
        } else {
            odd();
        }

        std::thread::sleep(std::time::Duration::from_secs(2));
        i += 1;
    }
}

#[instrument(ret)]
fn even(mut arg0: i32, arg1: bool) -> i32 {
    info!("Even called");

    if arg1 {
        arg0 += 1;
        arg0
    } else {
        0
    }
}

#[instrument]
fn odd() {
    info!("Odd called");
}
