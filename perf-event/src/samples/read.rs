use bytes::Buf;

use super::{Parse, ParseConfig, ReadValue, RecordEvent};

/// READ events happen when the kernel records the counters on its own.
///
/// This only happens when [`inherit_stat`] is enabled.
///
/// This struct corresponds to `PERF_RECORD_READ`. See the [manpage] for more
/// documentation.
///
/// [`inherit_stat`]: crate::Builder::inherit_stat
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct Read {
    pub pid: u32,
    pub tid: u32,
    pub values: ReadValue,
}

impl Parse for Read {
    fn parse<B: Buf>(config: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            pid: buf.get_u32_ne(),
            tid: buf.get_u32_ne(),
            values: ReadValue::parse(config, buf),
        }
    }
}

impl From<Read> for RecordEvent {
    fn from(read: Read) -> Self {
        Self::Read(read)
    }
}
