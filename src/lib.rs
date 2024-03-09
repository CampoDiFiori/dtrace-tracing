mod bindings;

use std::ffi::CString;

use tracing::{field::Visit, span};

use crate::bindings::*;

pub struct USDTTracingLayer;

impl Default for USDTTracingLayer {
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

impl<S> tracing_subscriber::Layer<S> for USDTTracingLayer
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if unsafe {
            [
                tracing_event_enabled(),
                tracing_trace_enabled(),
                tracing_debug_enabled(),
                tracing_info_enabled(),
                tracing_warn_enabled(),
                tracing_error_enabled(),
            ]
            .iter()
            .all(|&e| e == 0)
        } {
            return;
        }
        dbg!("running");

        let mut rv = RecordVisitor::new();
        event.record(&mut rv);

        event.metadata().level();

        unsafe {
            let n = CString::new(event.metadata().name()).unwrap();
            let c = CString::new(rv.fields()).unwrap();
            let m = CString::new(rv.message()).unwrap();

            let n_ptr = n.into_raw();
            let m_ptr = m.into_raw();
            let c_ptr = c.into_raw();

            if tracing_event_enabled() == 1 {
                tracing_event(n_ptr, m_ptr, c_ptr);
            }
            if tracing_trace_enabled() == 1 {
                tracing_trace(n_ptr, m_ptr, c_ptr);
            }
            if tracing_debug_enabled() == 1 {
                tracing_debug(n_ptr, m_ptr, c_ptr);
            }
            if tracing_info_enabled() == 1 {
                tracing_info(n_ptr, m_ptr, c_ptr);
            }
            if tracing_warn_enabled() == 1 {
                tracing_warn(n_ptr, m_ptr, c_ptr);
            }
            if tracing_error_enabled() == 1 {
                tracing_error(n_ptr, m_ptr, c_ptr);
            }

            _ = CString::from_raw(n_ptr);
            _ = CString::from_raw(m_ptr);
            _ = CString::from_raw(c_ptr);
        }
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        _id: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if unsafe { tracing_enter_enabled() } == 0 {
            return;
        }

        let mut rv = RecordVisitor::new();
        attrs.record(&mut rv);
        unsafe {
            let n = CString::new(attrs.metadata().name()).unwrap();
            let c = CString::new(rv.fields()).unwrap();

            let n_ptr = n.into_raw();
            let c_ptr = c.into_raw();

            tracing_enter(n_ptr, c_ptr);

            _ = CString::from_raw(n_ptr);
            _ = CString::from_raw(c_ptr);
        }
    }

    fn on_exit(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if unsafe { tracing_exit_enabled() } == 0 {
            return;
        }

        let mut rv = RecordVisitor::new();
        attrs.record(&mut rv);

        unsafe {
            let n = CString::new(attrs.metadata().name()).unwrap();
            let c = CString::new(rv.fields()).unwrap();

            let n_ptr = n.into_raw();
            let c_ptr = c.into_raw();

            tracing_exit(n_ptr, c_ptr);

            _ = CString::from_raw(n_ptr);
            _ = CString::from_raw(c_ptr);
        }
    }
}
