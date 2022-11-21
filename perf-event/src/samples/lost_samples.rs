use super::{Parse, RecordEvent};

/// LOST_SAMPLES records indicate that some samples were lost while using
/// hardware sampling.
///
/// This struct corresponds to `PERF_RECORD_LOST_SAMPLES`. See the [manpage]
/// for more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct LostSamples {
    /// The number of potentially lost samples.
    pub lost: u64,
}

impl Parse for LostSamples {
    fn parse<B: bytes::Buf>(_: &super::ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            lost: buf.get_u64_ne(),
        }
    }
}

impl From<LostSamples> for RecordEvent {
    fn from(lost: LostSamples) -> Self {
        Self::LostSamples(lost)
    }
}
