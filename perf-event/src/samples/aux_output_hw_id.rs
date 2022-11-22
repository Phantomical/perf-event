use super::{Parse, RecordEvent};

/// AUX_OUTPUT_HW_ID events allow matching data written to the aux area with
/// an architecture-specific hadrware ID.
///
/// This is needed when combining Intel PT along with sampling multiple PEBS
/// events. See the docs within `perf_event.h` for more explanation.
///
/// This struct corresponds to `PERF_RECORD_AUX_OUTPUT_HW_ID`. At the time of
/// writing it is not yet documented in the [manpage]. However, there is
/// documentation present within [the kernel source][src].
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
/// [src]: https://sourcegraph.com/github.com/torvalds/linux@eb7081409f94a9a8608593d0fb63a1aa3d6f95d8/-/blob/tools/include/uapi/linux/perf_event.h?L1205
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub struct AuxOutputHwId {
    pub hw_id: u64,
}

impl Parse for AuxOutputHwId {
    fn parse<B: bytes::Buf>(_: &super::ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            hw_id: buf.get_u64_ne(),
        }
    }
}

impl From<AuxOutputHwId> for RecordEvent {
    fn from(evt: AuxOutputHwId) -> Self {
        Self::AuxOutputHwId(evt)
    }
}
