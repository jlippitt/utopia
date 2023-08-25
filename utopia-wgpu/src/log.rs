#[cfg(not(debug_assertions))]
pub use release::init;

#[cfg(debug_assertions)]
pub use debug::init;

const ENV_VAR_NAME: &str = "LOG_LEVEL";

#[cfg(not(debug_assertions))]
mod release {
    use super::ENV_VAR_NAME;
    use std::{fmt, io};
    use tracing::event::Event;
    use tracing::subscriber::{DefaultGuard, Subscriber};
    use tracing_subscriber::filter::{EnvFilter, LevelFilter};
    use tracing_subscriber::fmt::format::{FormatEvent, FormatFields, Writer};
    use tracing_subscriber::fmt::FmtContext;
    use tracing_subscriber::registry::LookupSpan;

    struct Formatter;

    impl<S, N> FormatEvent<S, N> for Formatter
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
        N: for<'a> FormatFields<'a> + 'static,
    {
        fn format_event(
            &self,
            ctx: &FmtContext<'_, S, N>,
            mut writer: Writer<'_>,
            event: &Event<'_>,
        ) -> fmt::Result {
            ctx.format_fields(writer.by_ref(), event)?;
            writeln!(writer)
        }
    }

    pub fn init() -> io::Result<DefaultGuard> {
        let env_filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .with_env_var(ENV_VAR_NAME)
            .from_env_lossy();

        let subscriber = tracing_subscriber::fmt()
            .event_format(Formatter)
            .with_env_filter(env_filter)
            .with_writer(io::stdout)
            .finish();

        let guard = tracing::subscriber::set_default(subscriber);

        Ok(guard)
    }
}

#[cfg(debug_assertions)]
mod debug {
    use super::ENV_VAR_NAME;
    use std::collections::HashMap;
    use std::env;
    use std::fmt::Debug;
    use std::fs::{self, File};
    use std::io::{self, BufWriter, Write};
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use tracing::field::{Field, Visit};
    use tracing::span::{Attributes, Id, Record};
    use tracing::subscriber::DefaultGuard;
    use tracing::{Event, Level, Metadata, Subscriber};

    const DEFAULT_KEY: &str = "main";
    const LOG_BUFFER_SIZE: usize = 262144;

    fn create_writer(name: &str) -> io::Result<BufWriter<File>> {
        let file = File::create(format!("./log/{}.log", name))?;
        let writer = BufWriter::with_capacity(LOG_BUFFER_SIZE, file);
        Ok(writer)
    }

    struct FieldVisitor<'a> {
        writer: &'a mut BufWriter<File>,
    }

    impl<'a> FieldVisitor<'a> {
        pub fn new(writer: &'a mut BufWriter<File>) -> Self {
            Self { writer }
        }
    }

    impl<'a> Visit for FieldVisitor<'a> {
        fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
            if field.name() == "message" {
                writeln!(self.writer, "{:?}", value).unwrap();
            }
        }
    }

    struct LogRouter {
        writers: Vec<BufWriter<File>>,
        writer_map: HashMap<&'static str, usize>,
        stack: Vec<usize>,
    }

    impl LogRouter {
        pub fn new() -> io::Result<Self> {
            fs::create_dir_all("./log")?;

            let default_writer = create_writer(DEFAULT_KEY)?;

            let mut writer_map = HashMap::new();
            writer_map.insert(DEFAULT_KEY, 0);

            Ok(Self {
                writers: vec![default_writer],
                writer_map,
                stack: vec![0],
            })
        }

        pub fn new_span(&mut self, span: &Attributes<'_>) -> Id {
            let name = span.metadata().name();

            let writer_id = self.writer_map.get(name).copied().unwrap_or_else(|| {
                let writer_id = self.writers.len();
                let writer = create_writer(name).unwrap();
                self.writers.push(writer);
                self.writer_map.insert(name, writer_id);
                writer_id
            });

            // All span IDs are offset by 1, as 0 is reserved for the top-level writer
            Id::from_u64(writer_id as u64 + 1)
        }

        pub fn event(&mut self, event: &Event<'_>) {
            let writer_id = self.stack[self.stack.len() - 1];
            let writer = &mut self.writers[writer_id];
            event.record(&mut FieldVisitor::new(writer));
        }

        pub fn enter(&mut self, span: &Id) {
            self.stack.push(span.into_u64() as usize - 1);
        }

        pub fn exit(&mut self, span: &Id) {
            assert!(self.stack[self.stack.len() - 1] == span.into_u64() as usize - 1);
            self.stack.pop();
        }
    }

    struct DebugSubscriber {
        level: Level,
        router: Arc<Mutex<LogRouter>>,
    }

    impl DebugSubscriber {
        pub fn new() -> io::Result<Self> {
            let env_value = env::var(ENV_VAR_NAME).unwrap_or(String::new());
            let level = Level::from_str(&env_value).unwrap_or(Level::DEBUG);
            let router = Arc::new(Mutex::new(LogRouter::new()?));
            Ok(Self { level, router })
        }
    }

    impl Subscriber for DebugSubscriber {
        fn enabled(&self, metadata: &Metadata<'_>) -> bool {
            metadata.level() <= &self.level
        }

        fn new_span(&self, span: &Attributes<'_>) -> Id {
            self.router.try_lock().unwrap().new_span(span)
        }

        fn record(&self, _span: &Id, _values: &Record<'_>) {
            // Nothing
        }

        fn record_follows_from(&self, _span: &Id, _follows: &Id) {
            // Nothing
        }

        fn event(&self, event: &Event<'_>) {
            self.router.try_lock().unwrap().event(event)
        }

        fn enter(&self, span: &Id) {
            self.router.try_lock().unwrap().enter(span)
        }

        fn exit(&self, span: &Id) {
            self.router.try_lock().unwrap().exit(span)
        }
    }

    pub fn init() -> io::Result<DefaultGuard> {
        let subscriber = DebugSubscriber::new()?;
        let guard = tracing::subscriber::set_default(subscriber);
        Ok(guard)
    }
}
