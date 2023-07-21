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
        .with_env_var("LOG_LEVEL")
        .from_env_lossy();

    let subscriber = tracing_subscriber::fmt()
        .event_format(Formatter)
        .with_env_filter(env_filter)
        .with_writer(io::stdout)
        .finish();

    let guard = tracing::subscriber::set_default(subscriber);

    Ok(guard)
}
