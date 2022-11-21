use perf_event_open_sys::bindings;

use super::{Parse, ParseBuf, RecordEvent};

/// BPF_EVENT records indicate when a BPF program is loaded or unloaded.
///
/// This struct corresponds to `PERF_RECORD_BPF_EVENT`. See the [manpage] for
/// more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub struct BpfEvent {
    pub ty: BpfEventType,
    pub flags: u16,
    pub id: u32,
    pub tag: [u8; 8],
}

enum_binding! {
    /// Indicates the type of a [`BpfEvent`]
    pub struct BpfEventType : u16 {
        #[allow(missing_docs)]
        const UNKNOWN = bindings::PERF_BPF_EVENT_UNKNOWN as _;

        /// A BPF program was loaded.
        const PROG_LOAD = bindings::PERF_BPF_EVENT_PROG_LOAD as _;

        /// A BPF program was unloaded.
        const PROG_UNLOAD = bindings::PERF_BPF_EVENT_PROG_UNLOAD as _;
    }
}

impl Parse for BpfEvent {
    fn parse<B: bytes::Buf>(_: &super::ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            ty: BpfEventType(buf.get_u16_ne()),
            flags: buf.get_u16_ne(),
            id: buf.get_u32_ne(),
            tag: buf.parse_bytes(),
        }
    }
}

impl From<BpfEvent> for RecordEvent {
    fn from(bpf: BpfEvent) -> Self {
        Self::BpfEvent(bpf)
    }
}
