use bitflags::bitflags;
use perf_event_open_sys::bindings;

use super::{Parse, RecordEvent};

/// AUX records indicate that new data is available in the aux buffer region.
///
/// This struct corresponds to `PERF_RECORD_AUX`. See the [manpage] for more
/// documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct Aux {
    pub aux_offset: u64,
    pub aux_size: u64,
    pub flags: AuxFlags,
}

bitflags! {
    /// Flags describing the aux buffer update.
    ///
    /// Some flags are documented in the [manpage], others are not yet
    /// documented in the manpage but are instead documented in the [source].
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    /// [source]: https://sourcegraph.com/github.com/torvalds/linux@eb7081409f94a9a8608593d0fb63a1aa3d6f95d8/-/blob/tools/include/uapi/linux/perf_event.h?L1248
    pub struct AuxFlags : u64 {
        /// The data returned was truncated to fit within the buffer size.
        const TRUNCATED = bindings::PERF_AUX_FLAG_TRUNCATED as _;

        /// The data returned overwrote previous data.
        const OVERWRITE = bindings::PERF_AUX_FLAG_OVERWRITE as _;

        /// The record contains gaps.
        const PARTIAL = bindings::PERF_AUX_FLAG_PARTIAL as _;

        /// The aux sample collided with another.
        const COLLISION = bindings::PERF_AUX_FLAG_COLLISION as _;

        /// Certain bits actually contain a [`AuxPmuFormat`] enum.
        const PMU_FORMAT_MASK = bindings::PERF_AUX_FLAG_PMU_FORMAT_TYPE_MASK as _;
    }
}

enum_binding! {
    /// PMU-specific trace format type
    pub struct AuxPmuFormatType : u8 {
        const CORESIGHT = (bindings::PERF_AUX_FLAG_CORESIGHT_FORMAT_CORESIGHT >> 8) as _;
        const CORESIGHT_RAW = (bindings::PERF_AUX_FLAG_CORESIGHT_FORMAT_RAW >> 8) as _;
    }
}

impl AuxFlags {
    /// Create a new set of AuxFlags from the underlying bits.
    pub fn new(bits: u64) -> Self {
        Self { bits }
    }

    /// PMU-specific trace format type.
    pub fn pmu_format_type(&self) -> AuxPmuFormatType {
        AuxPmuFormatType(((*self & Self::PMU_FORMAT_MASK).bits() >> 8) as u8)
    }
}

impl Parse for Aux {
    fn parse<B: bytes::Buf>(_: &super::ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            aux_offset: buf.get_u64_ne(),
            aux_size: buf.get_u64_ne(),
            flags: AuxFlags::new(buf.get_u64_ne()),
        }
    }
}

impl From<Aux> for RecordEvent {
    fn from(aux: Aux) -> Self {
        Self::Aux(aux)
    }
}
