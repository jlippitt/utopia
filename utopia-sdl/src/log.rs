use std::fmt;
use tracing::event::Event;
use tracing::subscriber::{DefaultGuard, Subscriber};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt::{
    format::{FormatEvent, FormatFields, Writer},
    FmtContext, MakeWriter,
};
use tracing_subscriber::registry::LookupSpan;

pub struct Formatter;

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

pub fn set_subscriber<W>(writer: W) -> DefaultGuard
where
    W: for<'writer> MakeWriter<'writer> + 'static + Send + Sync,
{
    #[cfg(debug_assertions)]
    let default_level = LevelFilter::DEBUG;

    #[cfg(not(debug_assertions))]
    let default_level = LevelFilter::INFO;

    let env_filter = EnvFilter::builder()
        .with_default_directive(default_level.into())
        .from_env_lossy();

    let subscriber = tracing_subscriber::fmt()
        .event_format(Formatter)
        .with_env_filter(env_filter)
        .with_writer(writer)
        .finish();

    tracing::subscriber::set_default(subscriber)
}

#[cfg(debug_assertions)]
mod debug {
    use std::fs::{self, File};

    use std::io::{self, BufWriter};

    use std::sync::Mutex;

    const LOG_BUFFER_SIZE: usize = 262144;

    pub type DebugWriter = Mutex<BufWriter<File>>;

    pub fn create_debug_writer(name: &str) -> io::Result<DebugWriter> {
        fs::create_dir_all("./log")?;
        let file = File::create(format!("./log/{}.log", name))?;
        let buf_writer = BufWriter::with_capacity(LOG_BUFFER_SIZE, file);
        Ok(Mutex::new(buf_writer))
    }
}

#[cfg(debug_assertions)]
pub use debug::*;
