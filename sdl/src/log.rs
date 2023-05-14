use std::fmt;
use tracing::event::Event;
use tracing::subscriber::Subscriber as TracingSubscriber;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt::{
    format::{DefaultFields, FormatEvent, FormatFields, Writer},
    FmtContext,
    MakeWriter,
    Subscriber as FmtSubscriber,
};
use tracing_subscriber::registry::LookupSpan;

pub type Subscriber<W> = FmtSubscriber<DefaultFields, Formatter, EnvFilter, W>; 

pub struct Formatter;

impl<S, N> FormatEvent<S, N> for Formatter
where
    S: TracingSubscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(&self, ctx: &FmtContext<'_, S, N>, mut writer: Writer<'_>, event: &Event<'_>) -> fmt::Result {
        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

pub fn create_subscriber<W>(writer: W) -> Subscriber<W>
where
    W: for<'writer> MakeWriter<'writer> + 'static + Send + Sync
{
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .event_format(Formatter)
        .with_env_filter(env_filter)
        .with_writer(writer)
        .finish()
}