use super::{Parse, RecordEvent};

/// ITRACE_START records indicate when a process has started an instruction
/// trace.
///
/// This struct corresponds to `PERF_RECORD_ITRACE_START`. See the [manpage]
/// for more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct ITraceStart {
    /// Process ID of thread starting an instruction trace.
    pub pid: u32,

    /// Thread ID of thread starting an instruction trace.
    pub tid: u32,
}

impl Parse for ITraceStart {
    fn parse<B: bytes::Buf>(_: &super::ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            pid: buf.get_u32_ne(),
            tid: buf.get_u32_ne(),
        }
    }
}

impl From<ITraceStart> for RecordEvent {
    fn from(its: ITraceStart) -> Self {
        Self::ITraceStart(its)
    }
}
