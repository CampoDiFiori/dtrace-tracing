use std::ffi::CString;

use tracing::{field::Visit, info, instrument, span};
use tracing_subscriber::{filter, prelude::__tracing_subscriber_SubscriberExt, Layer};

extern "C" {
    pub fn tracing_provider_trace(arg1: *mut ::std::os::raw::c_char);
    pub fn tracing_provider_enter(arg1: *mut ::std::os::raw::c_char);
}

pub struct U;

impl Default for U {
    fn default() -> Self {
        Self
    }
}

struct RecordVisitor<'a> {
    s: &'a mut String,
}

impl Visit for RecordVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.s.push_str(&format!("{value:?}"));
        }
    }
}

impl<'a> RecordVisitor<'a> {
    pub fn new(s: &'a mut String) -> Self {
        Self { s }
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
        let mut s1 = "Hello from tracing: ".to_string();
        let mut rv = RecordVisitor::new(&mut s1);
        event.record(&mut rv);
    }

    fn on_layer(&mut self, subscriber: &mut S) {
        let _ = subscriber;
        // usdt::register_probes().unwrap();
    }

    fn on_enter(&self, _id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // let Some(s1) = ctx.span(id) else {
        //     return;
        // };

        unsafe {
            let c = CString::new("hello").unwrap();
            tracing_provider_trace(c.into_raw());
        }
    }

    fn on_exit(&self, _id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        unsafe {
            let c = CString::new("bye").unwrap();
            tracing_provider_enter(c.into_raw());
        }
    }
}

pub fn main() {
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter::LevelFilter::WARN))
        .with(U);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let mut i = 0;
    loop {
        if i % 2 == 0 {
            even();
        } else {
            odd();
        }

        std::thread::sleep(std::time::Duration::from_secs(2));
        i += 1;
    }
}

#[instrument]
fn even() {
    info!("Even called");
    // probes::trace!(|| "Stuff");
}

#[instrument]
fn odd() {
    info!("Odd called");
}
