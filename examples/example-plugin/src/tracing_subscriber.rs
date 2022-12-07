use tracing::*;

pub fn init() {
    subscriber::set_global_default(ExampleSubscriber::default()).unwrap();
}

/// This is a basic tracing Subscriber that forwards events as log messages via `log()`.
#[derive(Default)]
struct ExampleSubscriber {
    next_id: std::sync::atomic::AtomicUsize,
}

impl Subscriber for ExampleSubscriber {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _attrs: &span::Attributes<'_>) -> Id {
        Id::from_u64(self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u64)
    }

    fn record(&self, _span: &Id, _values: &span::Record<'_>) {
    }

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {
    }

    fn event(&self, event: &Event<'_>) {
        let location = match (event.metadata().file(), event.metadata().line()) {
            (Some(file), Some(line)) => {
                format!("{}:{}: ", file, line)
            }
            (Some(file), None) => {
                format!("{}: ", file)
            }
            _ => "".to_owned()
        };

        let mut v = Visitor { message: None };
        event.record(&mut v);
        crate::log(format!("{}{}", location, v.message.unwrap_or_else(|| "[Empty message]".to_owned())));
    }

    fn enter(&self, _span: &Id) {
    }

    fn exit(&self, _span: &Id) {
    }
}

struct Visitor {
    message: Option<String>,
}

impl field::Visit for Visitor {
    fn record_debug(&mut self, field: &field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        }
    }
}
